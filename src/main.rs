mod config;
mod github;
mod wizard;

use adw::prelude::*;
use github::{test, wait};
use gtk::glib;
use base64::Engine;
use adw::{Application, ApplicationWindow as AdwApplicationWindow};
use gtk::{Button, Entry, Label, Orientation, TextView, Picture};
use gtk::Box as GtkBox;
use hostname;
use mac_address::get_mac_address;
use octocrab::auth::DeviceCodes;
use octocrab::Octocrab;
use crate::{
    config::{SETTINGS, load_config},
    wizard::WizardData,
    github::create_github_issue
};
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    // Load config at startup
    SETTINGS.get_or_init(load_config);
    
    let settings = SETTINGS.get().unwrap();
    
    let app = Application::builder()
        .application_id(&settings.window.application_id)
        .build();

    // Initialize libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    app.connect_activate(build_ui);
    app.run();
}


fn build_ui(app: &Application) {
    let settings = SETTINGS.get().unwrap();
    let window = AdwApplicationWindow::builder()
        .application(app)
        .default_width(settings.window.default_width)
        .default_height(settings.window.default_height)
        .build();

    // Create header bar
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&adw::WindowTitle::new("Bug Report Wizard", "")));

    // Apply default styling
    window.add_css_class("default-spacing");

    let main_box = GtkBox::new(Orientation::Vertical, 0);
    main_box.append(&header);

    let wizard_data = Rc::new(RefCell::new(WizardData::default()));
    
    // Get system information
    let mac = match get_mac_address() {
        Ok(Some(addr)) => addr.to_string(),
        _ => "Unable to get MAC address".to_string(),
    };
    
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let box_ = GtkBox::new(Orientation::Vertical, 10);
    box_.set_margin_top(10);
    box_.set_margin_bottom(10);
    box_.set_margin_start(10);
    box_.set_margin_end(10);
    box_.set_vexpand(true);

    // System info display
    let system_info = Label::new(Some(&format!("MAC Address: {}\nHostname: {}", mac, hostname)));
    system_info.set_margin_bottom(10);
    box_.append(&system_info);

    // Step indicator
    let step_label = Label::new(Some("Step 1 of 3: Enter Your Name"));
    box_.append(&step_label);

    // Content box
    let content_box = GtkBox::new(Orientation::Vertical, 5);
    box_.append(&content_box);

    // Entry field (will be switched to TextView for description step)
    let entry = Entry::new();
    let text_view = TextView::new();
    text_view.set_wrap_mode(gtk::WrapMode::WordChar);
    text_view.set_visible(false);
    text_view.set_height_request(100);
    text_view.set_vexpand(true);
    content_box.append(&entry);
    content_box.append(&text_view);

    // Attachment buttons
    let attachment_box = GtkBox::new(Orientation::Horizontal, 5);
    let clipboard_button = Button::with_label("Paste Screenshot");
    attachment_box.append(&clipboard_button);
    content_box.append(&attachment_box);

    // Attachment status and preview
    let attachment_label = Label::new(None);
    let preview_picture = Picture::new();
    preview_picture.set_visible(false);
    preview_picture.set_size_request(200, 200);
    preview_picture.set_can_shrink(true);
    preview_picture.set_keep_aspect_ratio(true);
    content_box.append(&attachment_label);
    content_box.append(&preview_picture);

    // Navigation buttons
    let button_box = GtkBox::new(Orientation::Horizontal, 5);
    button_box.set_halign(gtk::Align::End);
    let back_button = Button::with_label("Back");
    let next_button = Button::with_label("Next");
    button_box.append(&back_button);
    button_box.append(&next_button);
    box_.append(&button_box);

    // Status label
    let status_label = Label::new(Some(""));
    box_.append(&status_label);

    back_button.set_sensitive(false);

    let update_ui = {
        let entry_clone = entry.clone();
        let text_view_clone = text_view.clone();
        let wizard_data = wizard_data.clone();
        let step_label = step_label.clone();
        let back_button = back_button.clone();
        let next_button = next_button.clone();
        let status_label = status_label.clone();
        let attachment_label = attachment_label.clone();
        Rc::new(glib::clone!(@strong entry_clone, @strong text_view_clone, @strong wizard_data, @strong step_label, @strong back_button, @strong next_button, @strong status_label, @strong preview_picture => move || {
            let data = wizard_data.borrow();
            match data.current_step {
                0 => {
                    step_label.set_text("Step 1 of 3: Enter Your Name");
                    entry_clone.set_text(&data.name);
                    back_button.set_sensitive(false);
                    next_button.set_label("Next");
                    println!("HERE maybe??!!1");
                }
                1 => {
                    step_label.set_text("Step 2 of 3: Enter Your Email");
                    entry_clone.set_text(&data.name);
                    back_button.set_sensitive(true);
                    next_button.set_label("Next");
                }
                2 => {
                    step_label.set_text("Step 3 of 3: Describe the Bug");
                    entry_clone.set_visible(false);
                    text_view_clone.set_visible(true);
                    text_view_clone.buffer().set_text(&data.description);
                    back_button.set_sensitive(true);
                    next_button.set_label("Submit");
                }
                3 => {
                    // Final summary screen
                    println!("HERE maybe??!!");
                    back_button.set_visible(false);
                    next_button.set_label("Quit");
                    next_button.set_visible(true);
                    
                    // Create and show summary text
                    let summary_text = format!(
                        "Report Summary\n\nName: {}\nEmail: {}\n\nDescription:\n{}",
                        data.name,
                        data.email,
                        data.description,
                    );
                    
                    // Configure and show status label
                    status_label.set_text(&data.name);
                    status_label.set_visible(true);
                    
                    // Configure and show status label first
                    status_label.set_margin_top(30);
                    status_label.set_margin_bottom(30);
                    status_label.set_margin_start(20);
                    status_label.set_margin_end(20);
                    status_label.set_wrap(true);
                    status_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
                    status_label.set_justify(gtk::Justification::Left);
                    status_label.set_halign(gtk::Align::Start);

                    // Then show attachment and preview if present
                    if data.attachment.is_some() {
                        attachment_label.set_text("Attachment included:");
                        attachment_label.set_visible(true);
                        preview_picture.set_visible(true);
                    } else {
                        attachment_label.set_visible(false);
                        preview_picture.set_visible(false);
                    }
                }
                _ => {}
            }
            if data.current_step != 3 {
                status_label.set_text("");
            }
        }))
    };

    // Make content elements initially visible
    content_box.set_visible(true);
    attachment_box.set_visible(true);
    attachment_label.set_visible(true);
    back_button.set_visible(true);
    
    update_ui();

    // Handle clipboard button
    clipboard_button.connect_clicked(glib::clone!(@strong wizard_data, @strong window, @strong attachment_label, @strong preview_picture => move |_| {
        let clipboard = gtk::prelude::WidgetExt::display(&window).clipboard();
        
        // Use glib::spawn_future for async clipboard access
        glib::spawn_future_local(glib::clone!(@strong wizard_data, @strong attachment_label, @strong preview_picture => async move {
            if let Ok(Some(texture)) = clipboard.read_texture_future().await {
                let width = texture.width() as usize;
                let height = texture.height() as usize;
                let stride = width * 4; // 4 bytes per pixel (RGBA)
                let mut rgba_buffer = vec![0u8; stride * height];
                texture.download(&mut rgba_buffer, stride);
                
                // Convert RGBA to PNG
                let mut png_buffer = Vec::new();
                {
                    let mut encoder = png::Encoder::new(&mut png_buffer, width as u32, height as u32);
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    let mut writer = encoder.write_header().unwrap();
                    writer.write_image_data(&rgba_buffer).unwrap();
                    // Writer is dropped here, finishing PNG encoding
                }
                
                let paintable = gtk::gdk::MemoryTexture::new(
                    width as i32,
                    height as i32,
                    gtk::gdk::MemoryFormat::R8g8b8a8,
                    &glib::Bytes::from(&rgba_buffer),
                    stride
                );
                preview_picture.set_paintable(Some(&paintable));
                preview_picture.set_visible(true);
                
                let mut data = wizard_data.borrow_mut();
                data.attachment = Some(("screenshot.png".to_string(), png_buffer));
                attachment_label.set_text("Screenshot attached");
            } else {
                attachment_label.set_text("No image in clipboard");
                preview_picture.set_visible(false);
            }
        }));
    }));


    let next_clicked = {
        let wizard_data = wizard_data.clone();
        let entry = entry.clone();
        let status_label = status_label.clone();
        let update_ui = update_ui.clone();
        let step_label = step_label.clone();
        let content_box = content_box.clone();
        let attachment_box = attachment_box.clone();
        let attachment_label = attachment_label.clone();
        let preview_picture = preview_picture.clone();
        let back_button = back_button.clone();
        let next_button = next_button.clone();


        move || {
            let mut data = wizard_data.borrow_mut();
            let current_text = if data.current_step == 2 {
                text_view.buffer().text(&text_view.buffer().start_iter(), &text_view.buffer().end_iter(), false).to_string()
            } else {
                entry.text().to_string()
            };

            match data.current_step {
                0 => {
                    if current_text.trim().is_empty() {
                        status_label.set_text("Please enter your name");
                        return;
                    }
                    
                    let mut code: String = String::new();
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    println!("TEST");
                    /*let auth_task = mitosis::spawn(counter, |counter| {
                        test(&counter);
                    });*/
                    match rt.block_on(test()) {
                        Ok(c) => {
                            code = c;
                        },
                        Err(_e) => {}
                    };
                    println!("After Test");
                    
                    data.name = code;
                    data.current_step = 1;
                }
                1 => {
                    
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    println!("HEllooWWAIT!!!");
                    match rt.block_on(wait()) {
                        Ok(_) => {
                        },
                        Err(e) => {println!("{}",e);}
                    };
                    println!("After HEllooWWAIT!!!");
                    if !current_text.contains('@') {
                        status_label.set_text("Please enter a valid email");
                        return;
                    }
                    data.email = current_text;
                    data.current_step += 1;
                }
                2 => {
                    if current_text.trim().is_empty() {
                        status_label.set_text("Please describe the bug");
                        return;
                    }
                    data.description = current_text;
                    
                    // Get MAC address
                    let mac = match get_mac_address() {
                        Ok(Some(addr)) => addr.to_string(),
                        _ => "Unable to get MAC address".to_string(),
                    };

                    // Prepare email content with optional attachment
                    let mut email_body = format!(
                        "Bug Report\n\nFrom: {}\nEmail: {}\nMAC Address: {}\n\nDescription:\n{}",
                        data.name, data.email, mac, data.description
                    );

                    // Add attachment if present
                    if let Some((filename, bytes)) = &data.attachment {
                        let base64_data = base64::engine::general_purpose::STANDARD.encode(bytes);
                        email_body.push_str(&format!("\n\nAttachment: {}\nBase64 Data:\n{}", filename, base64_data));
                    }

                    // Create GitHub issue
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    match rt.block_on(create_github_issue(&email_body)) {
                        Ok(_) => {
                            // Show summary screen
                            step_label.set_text("Report Submitted Successfully!");
                            step_label.set_margin_top(20);
                            
                            // Hide input elements
                            entry.set_visible(false);
                            text_view.set_visible(false);
                            attachment_box.set_visible(false);
                            
                            // Create and show summary text with better formatting
                            let summary_text = format!(
                                "Report Summary\n\nName: {}\nEmail: {}\n\nDescription:\n{}\n{}",
                                data.name,
                                data.email,
                                data.description,
                                if data.attachment.is_some() { "\nAttachment: Included" } else { "" }
                            );

                            // Keep content box visible for the preview
                            content_box.set_visible(true);
                            
                            // First configure and show status label with the summary
                            status_label.set_text(&summary_text);
                            status_label.set_visible(true);
                            status_label.set_margin_top(30);
                            status_label.set_margin_bottom(30);
                            status_label.set_margin_start(20);
                            status_label.set_margin_end(20);
                            status_label.set_wrap(true);
                            status_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
                            status_label.set_justify(gtk::Justification::Left);
                            status_label.set_halign(gtk::Align::Start);
                            status_label.add_css_class("large-text");
                            status_label.set_opacity(1.0);
                            
                            // After showing summary, update attachment display if present
                            if data.attachment.is_some() {
                                attachment_label.set_text("Attachment included:");
                                attachment_label.set_visible(true);
                                preview_picture.set_visible(true);
                            } else {
                                attachment_label.set_visible(false);
                                preview_picture.set_visible(false);
                            }
                            
                            // Update state and UI
                            data.current_step = 3;
                            
                            // Change navigation buttons
                            back_button.set_visible(false);
                            next_button.set_label("Quit");
                            button_box.set_margin_top(10);
                            button_box.set_margin_end(10);
                            button_box.set_margin_bottom(10);
                            button_box.set_halign(gtk::Align::End);
                        }
                        Err(e) => {
                            status_label.set_text(&format!("Error sending report: {}", e));
                            return;
                        }
                    }
                }
                3 => {
                    // Quit handled by window.close() in the button click handler
                    return;
                }
                _ => {}
            }
            drop(data);
            (*update_ui)();
        }
    };

    let back_clicked = {
        let wizard_data = wizard_data.clone();
        let _entry = entry.clone();
        let update_ui = update_ui.clone();

        move || {
            let mut data = wizard_data.borrow_mut();
            match data.current_step {
                1 => {
                    data.current_step = 0;
                }
                2 => {
                    data.current_step = 1;
                }
                _ => {}
            }
            drop(data);
            (*update_ui)();
        }
    };

    next_button.connect_clicked(glib::clone!(@strong window, @strong wizard_data, @strong next_clicked => move |button| {
        if wizard_data.borrow().current_step == 3 && button.label().map_or(false, |l| l == "Quit") {
            window.close();
        } else {
            next_clicked();
        }
    }));
    back_button.connect_clicked(move |_| back_clicked());

    main_box.append(&box_);
    window.set_content(Some(&main_box));
    window.present();
}

