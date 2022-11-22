use bevy::prelude::App;

pub trait SendEvent<T> {
    fn dispatch(&mut self, event: T);
}

pub trait EventSet {
    fn add_events(app: &mut App);
}

pub trait AddEventSet {
    fn add_event_set<E: EventSet>(&mut self) -> &mut Self;
}

impl AddEventSet for App {
    fn add_event_set<E: EventSet>(&mut self) -> &mut Self {
        E::add_events(self);
        self
    }
}

macro_rules! event_set {
    ($name:ident {$($event:ident),+}) => {
        #[allow(non_snake_case)]
        #[derive(bevy::ecs::system::SystemParam)]
        pub struct $name<'w, 's> {
            $(
                $event: bevy::prelude::EventWriter<'w, 's, $event>
            ),+
        }

        impl<'w, 's> EventSet for $name<'w, 's> {
            fn add_events(app: &mut App) {
                $(
                    app.add_event::<$event>();
                )+
            }
        }

        $(
            impl<'w, 's> SendEvent<$event> for $name<'w, 's> {
                fn dispatch(&mut self, event: $event) {
                    self.$event.send(event);
                }
            }
        )+
    };
}

pub(crate) use event_set;
