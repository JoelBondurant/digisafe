rm -rf target
cargo build --target aarch64-linux-android --release
cargo build --target armv7-linux-androideabi --release
cargo build --target i686-linux-android --release
cargo build --target x86_64-linux-android --release

rm -rf ../app/src/main/jniLibs
mkdir -p ../app/src/main/jniLibs/arm64-v8a
mkdir -p ../app/src/main/jniLibs/armeabi-v7a
mkdir -p ../app/src/main/jniLibs/x86
mkdir -p ../app/src/main/jniLibs/x86_64

ln -sf $PWD/target/aarch64-linux-android/release/libdroid.so ../app/src/main/jniLibs/arm64-v8a/libdroid.so
ln -sf $PWD/target/armv7-linux-androideabi/release/libdroid.so ../app/src/main/jniLibs/armeabi-v7a/libdroid.so
ln -sf $PWD/target/i686-linux-android/release/libdroid.so ../app/src/main/jniLibs/x86/libdroid.so
ln -sf $PWD/target/x86_64-linux-android/release/libdroid.so ../app/src/main/jniLibs/x86_64/libdroid.so
