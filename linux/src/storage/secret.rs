use libc::{
	c_void, ftruncate, madvise, mmap, munmap, off_t, syscall, sysconf, SYS_memfd_secret,
	MADV_DONTDUMP, MAP_FAILED, MAP_SHARED, PROT_READ, PROT_WRITE, _SC_PAGESIZE,
};
use std::{
	fmt, io,
	ops::{Deref, DerefMut},
	os::unix::io::{AsRawFd, FromRawFd, OwnedFd},
	ptr, slice,
	sync::{
		atomic::{compiler_fence, AtomicUsize, Ordering},
		OnceLock,
	},
};

static PAGE_SIZE: OnceLock<usize> = OnceLock::new();

fn get_page_size() -> usize {
	*PAGE_SIZE.get_or_init(|| {
		let res = unsafe { sysconf(_SC_PAGESIZE) };
		if res < 0 {
			4096
		} else {
			res as usize
		}
	})
}

pub struct SecretMemory {
	secret_fd: OwnedFd,
	capacity: usize,
	length: AtomicUsize,
}

impl SecretMemory {
	pub fn new(capacity: usize) -> Result<Self, io::Error> {
		let fd = unsafe { syscall(SYS_memfd_secret, 0) as i32 };
		if fd < 0 {
			return Err(io::Error::last_os_error());
		}
		let owned_fd = unsafe { OwnedFd::from_raw_fd(fd) };
		if unsafe { ftruncate(owned_fd.as_raw_fd(), capacity as off_t) } < 0 {
			return Err(io::Error::last_os_error());
		}
		Ok(Self {
			secret_fd: owned_fd,
			capacity,
			length: AtomicUsize::new(0),
		})
	}

	pub fn new_pages(num_pages: usize) -> Result<Self, io::Error> {
		Self::new(num_pages * get_page_size())
	}

	pub fn view(&self) -> Result<MappedGuard<'_>, io::Error> {
		let mapped_ptr = unsafe {
			mmap(
				ptr::null_mut(),
				self.capacity,
				PROT_READ | PROT_WRITE,
				MAP_SHARED,
				self.secret_fd.as_raw_fd(),
				0,
			)
		};
		if mapped_ptr == MAP_FAILED {
			return Err(io::Error::last_os_error());
		}
		unsafe {
			madvise(mapped_ptr, self.capacity, MADV_DONTDUMP);
		}
		Ok(MappedGuard {
			mapped_ptr,
			capacity: self.capacity,
			length: &self.length,
		})
	}

	pub fn write(&self, offset: usize, data: &[u8]) -> Result<(), io::Error> {
		let mut guard = self.view()?;
		let end = offset
			.checked_add(data.len())
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Overflow"))?;
		if end > self.capacity {
			return Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				"Buffer overflow",
			));
		}
		guard[offset..end].copy_from_slice(data);
		self.length.fetch_max(end, Ordering::Release);
		Ok(())
	}

	pub fn read(&self) -> Result<MappedGuard<'_>, io::Error> {
		let mapped_ptr = unsafe {
			mmap(
				ptr::null_mut(),
				self.capacity,
				PROT_READ | PROT_WRITE,
				MAP_SHARED,
				self.secret_fd.as_raw_fd(),
				0,
			)
		};
		if mapped_ptr == MAP_FAILED {
			return Err(io::Error::last_os_error());
		}
		unsafe {
			madvise(mapped_ptr, self.capacity, MADV_DONTDUMP);
		}
		Ok(MappedGuard {
			mapped_ptr,
			capacity: self.len(),
			length: &self.length,
		})
	}

	pub fn len(&self) -> usize {
		self.length.load(Ordering::Acquire)
	}

	#[allow(dead_code)]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn zeroize(&self) -> Result<(), io::Error> {
		let mut guard = self.view()?;
		for byte in guard.iter_mut() {
			unsafe { ptr::write_volatile(byte, 0) };
		}
		compiler_fence(Ordering::SeqCst);
		self.length.store(0, Ordering::Release);
		Ok(())
	}
}

pub struct MappedGuard<'a> {
	mapped_ptr: *mut c_void,
	capacity: usize,
	length: &'a AtomicUsize,
}

impl<'a> MappedGuard<'a> {
	pub fn len(&self) -> usize {
		self.length.load(Ordering::Acquire)
	}

	#[allow(dead_code)]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

impl<'a> Deref for MappedGuard<'a> {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		unsafe { slice::from_raw_parts(self.mapped_ptr as *const u8, self.capacity) }
	}
}

impl<'a> DerefMut for MappedGuard<'a> {
	fn deref_mut(&mut self) -> &mut [u8] {
		unsafe { slice::from_raw_parts_mut(self.mapped_ptr as *mut u8, self.capacity) }
	}
}

impl<'a> Drop for MappedGuard<'a> {
	fn drop(&mut self) {
		unsafe {
			munmap(self.mapped_ptr, self.capacity);
		}
	}
}

impl<'a> fmt::Debug for MappedGuard<'a> {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		formatter
			.debug_struct("MappedGuard")
			.field("capacity", &self.capacity)
			.field("length", &self.len())
			.field("data", &"*** REDACTED ***")
			.finish()
	}
}

impl<'a> fmt::Display for MappedGuard<'a> {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			formatter,
			"SecretMemoryView(unaddressable, {}/{} bytes)",
			self.len(),
			self.capacity,
		)
	}
}
