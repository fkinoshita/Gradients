/* window.rs
 *
 * Copyright 2023 Felipe Kinoshita
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use glib::clone;

use gtk::{gdk, prelude::*};
use gtk::{gio, glib};

use adw::prelude::*;
use adw::subclass::prelude::*;

use crate::config::PROFILE;

#[derive(Debug, Copy, Clone)]
enum GradientType {
    Linear,
    Radial,
    Conic,
}

impl From<u32> for GradientType {
    fn from(value: u32) -> Self {
        match value {
            1 => GradientType::Radial,
            2 => GradientType::Conic,
            //default to Linear, including 0
            _ => GradientType::Linear,
        }
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/fkinoshita/Vibrant/window.ui")]
    pub struct VibrantWindow {
        // Template widgets
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub navigation_view: TemplateChild<adw::NavigationView>,

        #[template_child]
        pub gradient_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub gradient_overlay: TemplateChild<gtk::Overlay>,

        #[template_child]
        pub direction_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub gradient_combo: TemplateChild<adw::ComboRow>,

        #[template_child]
        pub color_one_button: TemplateChild<gtk::ColorDialogButton>,
        #[template_child]
        pub color_two_button: TemplateChild<gtk::ColorDialogButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VibrantWindow {
        const NAME: &'static str = "VibrantWindow";
        type Type = super::VibrantWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VibrantWindow {}
    impl WidgetImpl for VibrantWindow {}
    impl WindowImpl for VibrantWindow {}
    impl ApplicationWindowImpl for VibrantWindow {}
    impl AdwApplicationWindowImpl for VibrantWindow {}
}

glib::wrapper! {
    pub struct VibrantWindow(ObjectSubclass<imp::VibrantWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,        @implements gio::ActionGroup, gio::ActionMap;
}

impl VibrantWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        let win: VibrantWindow = glib::Object::builder()
            .property("application", application)
            .build();

        win.init();
        win.setup_signals();

        win
    }

    fn init(&self) {
        let imp = self.imp();

        if PROFILE == "Devel" {
            self.add_css_class("devel");
        }

        imp.color_one_button.set_rgba(&gdk::RGBA::BLUE);
        imp.color_two_button
            .set_rgba(&gdk::RGBA::new(1.0, 0.75, 0.8, 1.0));
        self.update_gradient();
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        imp.gradient_combo.connect_selected_item_notify(
            clone!(@strong self as this => move |combo| {
                //disable when gradient is radial, as it does effect anything
                this.imp().direction_combo.set_sensitive(combo.selected() != 1);
                this.update_gradient();
            }),
        );

        imp.direction_combo.connect_notify_local(
            Some("selected"),
            clone!(@strong self as this => move |_combo, _| {
                this.update_gradient();
            }),
        );

        imp.color_one_button
            .connect_rgba_notify(clone!(@strong self as this => move |_| {
                this.update_gradient();
            }));

        imp.color_two_button
            .connect_rgba_notify(clone!(@strong self as this => move |_| {
                this.update_gradient();
            }));
    }

    fn update_gradient(&self) {
        let imp = self.imp();
        let provider = gtk::CssProvider::new();

        let gradient_type = GradientType::from(imp.gradient_combo.selected());
        let degree = imp.direction_combo.selected() as u16 * 90;

        let gradient = match gradient_type {
            GradientType::Linear => format!("linear-gradient({}deg,", degree),
            GradientType::Radial => "radial-gradient(".to_owned(),
            GradientType::Conic => format!(
                "conic-gradient(from {}deg,",
                //adjust degree to only switch bottom and top direction
                degree + (degree % 180 == 0) as u16 * 180
            ),
        };

        let css = format!(
            ".gradient-box {{background: {} {}, {});}}",
            gradient,
            imp.color_one_button.rgba(),
            imp.color_two_button.rgba()
        );

        provider.load_from_data(css.as_str());

        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(&display, &provider, 1000);
        }
    }
}
