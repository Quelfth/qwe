use crate::{app::AppSignal, action::{Action, AppAction}};


impl<T> Action<T> for AppAction {
    fn act(self, _: T) -> Option<AppSignal> {
        match self {
            AppAction::Quit => Some(AppSignal::Quit),
        }
    }
}