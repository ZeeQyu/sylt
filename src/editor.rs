use crate::imports::*;

#[derive(Default)]
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugin(bevy_yoleck::bevy_egui::EguiPlugin)
            .add_plugin(bevy_yoleck::YoleckPluginForEditor)
            .add_plugin(bevy_yoleck::vpeol_2d::Vpeol2dPlugin)
            .add_plugin(SyncWithEditorState {
                when_editor: GameState::Editor,
                when_game: GameState::Game,
            })
            .add_system(immobilize_physics_bodies.in_set(OnUpdate(GameState::Editor)))
            .add_system(immobilize_physics_bodies.in_schedule(OnEnter(GameState::Editor)))
            .add_system(remobilize_physics_bodies.in_schedule(OnExit(GameState::Editor)))
        ;
    }
}

pub struct SyncWithEditorState<T>
    where
        T: 'static + Sync + Send + std::fmt::Debug + Clone + std::cmp::Eq + std::hash::Hash,
{
    pub when_editor: T,
    pub when_game: T,
}

impl<T> Plugin for SyncWithEditorState<T>
    where
        T: 'static + States + Sync + Send + std::fmt::Debug + Clone + std::cmp::Eq + std::hash::Hash,
{
    fn build(&self, app: &mut App) {
        app.add_state::<T>();
        let initial_state = self.when_editor.clone();
        app.add_startup_system(move |mut game_state: ResMut<NextState<T>>| {
            game_state.set(initial_state.clone());
        });
        let when_editor = self.when_editor.clone();
        let when_game = self.when_game.clone();
        app.add_system(
            move |editor_state: Res<State<YoleckEditorState>>,
                  mut game_state: ResMut<NextState<T>>| {
                game_state.set(match editor_state.0 {
                    YoleckEditorState::EditorActive => when_editor.clone(),
                    YoleckEditorState::GameActive => when_game.clone(),
                });
            },
        );
    }
}

#[derive(Component)]
struct ImmobilizedPhysicsBody {
    rigid_body: RigidBody,
    collider: Option<Collider>,
}

fn immobilize_physics_bodies(query: Query<(Entity, &RigidBody, Option<&Collider>)>, mut commands: Commands) {
    for (entity, rigid_body, collider) in query.iter() {
        commands.entity(entity).insert(ImmobilizedPhysicsBody {
            rigid_body: rigid_body.clone(),
            collider: collider.cloned(),
        }).remove::<RigidBody>().remove::<Collider>();
    }
}

fn remobilize_physics_bodies(query: Query<(Entity, &ImmobilizedPhysicsBody)>, mut commands: Commands) {
    for (entity, immobilized) in query.iter() {
        if let Some(collider) = &immobilized.collider {
            commands.entity(entity)
                .insert(immobilized.rigid_body)
                .insert(collider.clone())
                .remove::<ImmobilizedPhysicsBody>();
        } else {
            commands.entity(entity)
                .insert(immobilized.rigid_body)
                .remove::<ImmobilizedPhysicsBody>();
        }
    }
}


pub fn create_editor_object(commands: &mut Commands, writer: &mut EventWriter<YoleckEditorEvent>, yoleck: &mut ResMut<YoleckState>, type_name: &str, value: serde_json::Value) {
    let cmd = commands.spawn(YoleckRawEntry {
        header: YoleckEntryHeader {
            type_name: String::from(type_name),
            name: String::from(""),
        },
        data: value,
    });
    writer.send(YoleckEditorEvent::EntitySelected(cmd.id()));
    yoleck.entity_being_edited = Some(cmd.id());
    yoleck.level_needs_saving = true;
}

