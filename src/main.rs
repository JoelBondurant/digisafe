use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use gtk::prelude::*;

mod appdb;

fn main() {
    let app_id = "com.digisafe.db";
    let app = gtk::Application::builder().application_id(app_id).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {

    let db = Arc::new(RwLock::new(appdb::AppDB::new()));
    let status_bar = Rc::new(RefCell::new(gtk::Statusbar::new()));
    let side_margin = 20;

    let main_box = Rc::new(RefCell::new(gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build()));

    let key_entry = Rc::new(RefCell::new(gtk::Entry::builder()
        .margin_top(20)
        .margin_bottom(10)
        .margin_start(side_margin)
        .margin_end(side_margin)
        .max_length(64)
        .tooltip_text("Key Text")
        .build()));
    
    let val_entry = Rc::new(RefCell::new(gtk::TextView::builder()
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(side_margin)
        .margin_end(side_margin)
        .height_request(400)
        .tooltip_text("Value Text")
        .build()));

    let get_button = gtk::Button::builder()
        .label("Get")
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(10)
        .margin_end(20)
        .build();
    let db_get = Arc::clone(&db);
    let key_get = Rc::clone(&key_entry);
    let val_get = Rc::clone(&val_entry);
    let main_box2 = Rc::clone(&main_box);
    get_button.connect_clicked(move |_| {
        main_box2.borrow().set_sensitive(false);
        let key = key_get.borrow().text().to_string();
        if let Some(val) = db_get.write().unwrap().get(&key) {
            val_get.borrow_mut().buffer().set_text(&val);
        } else {
            val_get.borrow_mut().buffer().set_text("");
        }
        main_box2.borrow().set_sensitive(true);
    });
    get_button.set_size_request(140, 20);

    let set_button = gtk::Button::builder()
        .label("Set")
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(10)
        .margin_end(10)
        .build();
    let db_set = Arc::clone(&db);
    let key_set = Rc::clone(&key_entry);
    let val_set = Rc::clone(&val_entry);
    let main_box2 = Rc::clone(&main_box);
    set_button.connect_clicked(move |_| {
        main_box2.borrow().set_sensitive(false);
        let key = key_set.borrow().text().to_string();
        let bounds = val_set.borrow().buffer().bounds();
        let val = val_set.borrow().buffer().text(&bounds.0, &bounds.1, false).to_string();
        db_set.write().unwrap().set(key, val);
        main_box2.borrow().set_sensitive(true);
    });
    set_button.set_size_request(140, 20);

    let save_button = gtk::Button::builder()
        .label("Save")
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(10)
        .margin_end(20)
        .build();
    let (save_sender, save_receiver) = gtk::glib::MainContext::channel::<String>(gtk::glib::PRIORITY_DEFAULT);
    let status_bar2 = Rc::clone(&status_bar);
    let main_box2 = Rc::clone(&main_box);
    save_receiver.attach(None, move|msg| {
        status_bar2.borrow().push(0, &msg);
        main_box2.borrow().set_sensitive(true);
        gtk::glib::Continue(true)
    });
    let db_save = Arc::clone(&db);
    let main_box2 = Rc::clone(&main_box);
    save_button.connect_clicked(move |_| {
        main_box2.borrow().set_sensitive(false);
        let db_save = Arc::clone(&db_save);
        let save_sender = save_sender.clone();
        std::thread::spawn(move || {
            let msg = db_save.read().unwrap().save();
            save_sender.send(msg).expect("save sender error");
        });
    });
    save_button.set_size_request(140, 20);

    let button_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .halign(gtk::Align::Center)
        .build();
    main_box.borrow().append(&*key_entry.borrow());
    main_box.borrow().append(&*val_entry.borrow());
    button_box.append(&get_button);
    button_box.append(&set_button);
    button_box.append(&save_button);
    main_box.borrow().append(&button_box);

    status_bar.borrow().push(0, "locked");
    main_box.borrow().append(&*status_bar.borrow());

    main_box.borrow().set_sensitive(false);

    let window = Rc::new(gtk::ApplicationWindow::builder()
        .application(app)
        .default_width(800)
        .default_height(600)
        .title("DigiSafe")
        .child(&*main_box.borrow())
        .visible(true)
        .build());
    window.present();

    let (unlock_sender, unlock_receiver) = gtk::glib::MainContext::channel::<String>(gtk::glib::PRIORITY_DEFAULT);
    let window2 = Rc::clone(&window);
    let db2 = Arc::clone(&db);
    let main_box2 = Rc::clone(&main_box);
    gtk::glib::MainContext::default().spawn_local(unlock_dialog(window2, db2, unlock_sender));
    unlock_receiver.attach(None, move|msg| {
        status_bar.borrow().push(0, &msg);
        if msg == "unlocked" {
            main_box2.borrow().set_sensitive(true);
            gtk::glib::Continue(false)
        } else {
            gtk::glib::Continue(true)
        }
    });

    let window2 = Rc::clone(&window);
    gtk::glib::timeout_add_seconds_local(10, move|| { 
        window2.clipboard().set_text("");
        gtk::glib::Continue(true)
    });
}


async fn unlock_dialog<W: gtk::glib::IsA<gtk::Window>>(window: Rc<W>, db: Arc<RwLock<appdb::AppDB>>, sender: gtk::glib::Sender<String>) {
    let password_entry = gtk::PasswordEntry::builder()
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .width_request(300)
        .tooltip_text("Password")
        .show_peek_icon(true)
        .build();
    let unlock_button = gtk::Button::builder()
        .label("Unlock")
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(20)
        .margin_end(20)
        .build();
    let dialog_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Center)
        .build();
    dialog_box.append(&password_entry);
    dialog_box.append(&unlock_button);
    let dialog = Rc::new(gtk::Dialog::builder()
        .transient_for(&*window)
        .title("Enter Password")
        .default_height(100)
        .default_width(300)
        .modal(true)
        .child(&dialog_box)
        .build());
    let dialog_clone = Rc::clone(&dialog);
    let dbc = Arc::clone(&db);
    unlock_button.connect_clicked(move |_| {
        let raw_password = password_entry.text().to_string();
        let dbcc = Arc::clone(&dbc);
        let sender = sender.clone();
        std::thread::spawn(move || {
            dbcc.write().unwrap().set_password(raw_password);
            let msg = dbcc.write().unwrap().load();
            sender.send(msg).expect("unlock failure");
        });
        dialog_clone.close();
        });
    dialog.run_future().await;
}