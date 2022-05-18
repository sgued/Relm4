use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{
    factory::{DynamicIndex, FactoryComponent, FactoryVecDeque},
    gtk, ComponentParts, ComponentSender, RelmApp, Sender, SimpleComponent, WidgetPlus,
};

#[derive(Debug)]
struct Counter {
    value: u8,
}

#[derive(Debug)]
enum CounterMsg {
    Increment,
    Decrement,
}

enum CounterOutput {
    SendFront(DynamicIndex),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
}

struct CounterWidgets {
    label: gtk::Label,
}

impl FactoryComponent<gtk::Box, AppMsg> for Counter {
    type Widgets = CounterWidgets;

    type InitParams = u8;

    type Input = CounterMsg;
    type Output = CounterOutput;

    type Root = gtk::Box;
    type Command = ();
    type CommandOutput = ();

    fn output_to_parent_msg(output: Self::Output) -> Option<AppMsg> {
        Some(match output {
            CounterOutput::SendFront(index) => AppMsg::SendFront(index),
            CounterOutput::MoveUp(index) => AppMsg::MoveUp(index),
            CounterOutput::MoveDown(index) => AppMsg::MoveDown(index),
        })
    }

    fn init_root(&self) -> Self::Root {
        relm4::view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
            }
        }
        root
    }

    fn init_model(
        value: Self::InitParams,
        _index: &DynamicIndex,
        _input: &Sender<Self::Input>,
        _output: &Sender<Self::Output>,
    ) -> Self {
        Self { value }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &gtk::Widget,
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) -> Self::Widgets {
        relm4::view! {
            label = gtk::Label {
                set_label: &self.value.to_string(),
                set_width_chars: 3,
            }
        }

        relm4::view! {
            add_button = gtk::Button {
                set_label: "+",
                connect_clicked(input) => move |_| {
                    input.send(CounterMsg::Increment)
                }
            }
        }

        relm4::view! {
            remove_button = gtk::Button {
                set_label: "-",
                connect_clicked(input) => move |_| {
                    input.send(CounterMsg::Decrement)
                }
            }
        }

        relm4::view! {
            move_up_button = gtk::Button {
                set_label: "Up",
                connect_clicked(output, index) => move |_| {
                    output.send(CounterOutput::MoveUp(index.clone()))
                }
            }
        }

        relm4::view! {
            move_down_button = gtk::Button {
                set_label: "Down",
                connect_clicked(output, index) => move |_| {
                    output.send(CounterOutput::MoveDown(index.clone()))
                }
            }
        }

        relm4::view! {
            to_front_button = gtk::Button {
                set_label: "To start",
                connect_clicked(output, index) => move |_| {
                    output.send(CounterOutput::SendFront(index.clone()))
                }
            }
        }

        root.append(&label);
        root.append(&add_button);
        root.append(&remove_button);
        root.append(&move_up_button);
        root.append(&move_down_button);
        root.append(&to_front_button);

        CounterWidgets { label }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        _input: &Sender<Self::Input>,
        _ouput: &Sender<Self::Output>,
    ) -> Option<Self::Command> {
        match msg {
            CounterMsg::Increment => {
                self.value = self.value.wrapping_add(1);
            }
            CounterMsg::Decrement => {
                self.value = self.value.wrapping_sub(1);
            }
        }
        None
    }

    fn update_view(
        &self,
        widgets: &mut Self::Widgets,
        _input: &Sender<Self::Input>,
        _output: &Sender<Self::Output>,
    ) {
        widgets.label.set_label(&self.value.to_string());
    }
}

struct AppModel {
    created_widgets: u8,
    counters: FactoryVecDeque<gtk::Box, Counter, AppMsg>,
}

#[derive(Debug)]
enum AppMsg {
    AddCounter,
    RemoveCounter,
    SendFront(DynamicIndex),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
}

#[relm4::component]
impl SimpleComponent for AppModel {
    // AppWidgets is generated by the macro
    type Widgets = AppWidgets;

    type InitParams = u8;

    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Factory example"),
            set_default_width: 300,
            set_default_height: 100,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Button {
                    set_label: "Add counter",
                    connect_clicked(sender) => move |_| {
                        sender.input(AppMsg::AddCounter);
                    }
                },

                gtk::Button {
                    set_label: "Remove counter",
                    connect_clicked(sender) => move |_| {
                        sender.input(AppMsg::RemoveCounter);
                    }
                },

                append: counter_box = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,
                }
            }
        }
    }

    // Initialize the UI.
    fn init(
        counter: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Insert the macro codegen here
        let widgets = view_output!();

        let model = AppModel {
            created_widgets: counter,
            counters: FactoryVecDeque::new(widgets.counter_box.clone(), &sender.input),
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: &ComponentSender<Self>) {
        match msg {
            AppMsg::AddCounter => {
                self.counters.push_back(self.created_widgets);
                self.created_widgets = self.created_widgets.wrapping_add(1);
            }
            AppMsg::RemoveCounter => {
                self.counters.pop_back();
            }
            AppMsg::SendFront(index) => {
                self.counters.move_front(index.current_index());
            }
            AppMsg::MoveDown(index) => {
                let index = index.current_index();
                let new_index = index + 1;
                // Already at the end?
                if new_index < self.counters.len() {
                    self.counters.move_to(index, new_index);
                }
            }
            AppMsg::MoveUp(index) => {
                let index = index.current_index();
                // Already at the start?
                if index != 0 {
                    self.counters.move_to(index, index - 1);
                }
            }
        }

        self.counters.render_changes();
    }
}

fn main() {
    let app: RelmApp<AppModel> = RelmApp::new("relm4.test.factory");
    app.run(0);
}
