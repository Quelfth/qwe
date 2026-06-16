use crate::{app::AppSignal, action::{Action, NavigatorAction}, navigator::{NameBox, Navigator}};


impl Action<&mut Navigator> for NavigatorAction {
    fn act(self, nav: &mut Navigator) -> Option<AppSignal> {
        match self {
            NavigatorAction::Down => _= nav.navigate_down(),
            NavigatorAction::Up => _= nav.navigate_up(),
            NavigatorAction::Out => nav.navigate_out(),
            NavigatorAction::In => nav.navigate_in(),
            NavigatorAction::NewChild => nav.new_child(),
            NavigatorAction::NewSibling => nav.new_sibling(),
            NavigatorAction::Rename => nav.rename(),
            NavigatorAction::DeleteEmpty => nav.delete_empty(),
            NavigatorAction::Editor => return Some(AppSignal::Editor),
        }

        None
    }
}