use bevy::{prelude::*, utils::HashMap};

#[derive(Component, Reflect)]
pub struct OverheadLabel {
    pub current_task: String,
    pub plan: Vec<String>,
    pub coins: i32,
    pub mood: String,
    pub offset: Vec3,
}

impl std::fmt::Display for OverheadLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Mood::{}, {} coins\n───", self.mood, self.coins)?;
        let mut reached = false;
        for (i, task) in self.plan.iter().enumerate() {
            if !reached && self.current_task == *task {
                write!(f, "{}(running)", task)?;
                reached = true;
            } else if reached {
                write!(f, "{}", task)?;
            } else {
                write!(f, "{}(done)", task)?;
            }
            if i < self.plan.len() - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl OverheadLabel {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            current_task: text.into(),
            plan: vec![],
            coins: 0,
            mood: "".into(),
            offset: Vec3::new(-0.4, 13.8, 0.0),
        }
    }
}

pub fn label_plugin(app: &mut App) {
    app.add_systems(Update, spawn_label);
    app.add_systems(Update, update_labels);
    app.register_type::<OverheadLabel>();
    app.register_type::<ExampleLabel>();
    app.init_resource::<LabelMap>();
}

#[derive(Component, Reflect)]
struct ExampleLabel {
    entity: Entity,
    offset: Vec3,
}

#[derive(Resource, Default)]
struct LabelMap {
    e2label: HashMap<Entity, Entity>,
}

fn spawn_label(
    q: Query<(Entity, &OverheadLabel), Or<(Changed<OverheadLabel>, Added<OverheadLabel>)>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut label_map: ResMut<LabelMap>,
) {
    for (entity, label) in q.iter() {
        // delete any existing label for this entity:
        if let Some(e) = label_map.e2label.get(&entity) {
            commands.entity(*e).despawn_recursive();
        }
        // We need the full version of this font so we can use box drawing characters.
        let text_style = TextFont {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: 10.0,
            ..default()
        };
        let label_text_style = (
            text_style.clone(),
            TextColor(bevy::color::palettes::css::NAVY.into()),
        );
        let label_id = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    // flex_direction: FlexDirection::Column,
                    // align_items: AlignItems::FlexEnd,
                    overflow: Overflow::visible(),
                    ..default()
                },
                ExampleLabel {
                    entity,
                    offset: label.offset,
                },
                Name::new("Label"),
            ))
            .with_children(|parent| {
                // parent.spawn((
                //     Text::new("Test label"),
                //     label_text_style.clone(),
                //     TextLayout::new_with_justify(JustifyText::Center).with_no_wrap(),
                //     Node {
                //         position_type: PositionType::Relative,
                //         bottom: Val::ZERO,
                //         ..default()
                //     },
                // ));
                parent.spawn((
                    Text::new(add_stem(&label.to_string())),
                    label_text_style.clone(),
                    TextLayout::new_with_justify(JustifyText::Center).with_no_wrap(),
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::ZERO,
                        ..default()
                    },
                    // TextLayout::default().with_no_wrap(),
                ));
            })
            .id();
        label_map.e2label.insert(entity, label_id);
    }
}

fn add_stem(text: &str) -> String {
    format!("{text}\n─┬─\n│")
}

fn update_labels(
    mut q_labels: Query<(&mut Node, &ExampleLabel)>,
    q_labeled: Query<&GlobalTransform>,
    q_camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let (camera, camera_global_transform) = q_camera.into_inner();

    for (mut node, label) in &mut q_labels {
        let world_position = q_labeled.get(label.entity).unwrap().translation() + label.offset;

        let viewport_position = camera
            .world_to_viewport(camera_global_transform, world_position)
            .unwrap();

        node.top = Val::Px(viewport_position.y);
        node.left = Val::Px(viewport_position.x);
    }
}
