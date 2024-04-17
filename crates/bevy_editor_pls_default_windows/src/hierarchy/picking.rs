use bevy::{
    ecs::event::event_update_condition, prelude::*, render::render_resource::PrimitiveTopology,
};
use bevy_editor_pls_core::Editor;
use bevy_inspector_egui::{
    bevy_egui::EguiContexts, bevy_inspector::hierarchy::SelectionMode, egui::Pos2,
};
use bevy_mod_picking::{backends::egui::EguiPointer, prelude::*};

use crate::hierarchy::HierarchyWindow;

pub struct EditorPickingSet;

/// Prevents the entity from being selectable in the editor window.
#[derive(Component)]
pub struct NoEditorPicking;

pub fn setup(app: &mut App) {
    info!("Adding picking set");
    app.add_event::<PointerClick>()
        .add_plugins(DefaultPickingPlugins.build().disable::<EguiBackend>())
        .add_systems(
            Update,
            auto_add_editor_picking_set.run_if(requires_add_pickable),
        )
        .add_systems(
            Update,
            handle_events.run_if(event_update_condition::<PointerClick>),
        );
}

#[derive(Event, Debug)]
struct PointerClick(Entity, PointerButton, Vec2);

impl From<ListenerInput<Pointer<Click>>> for PointerClick {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        PointerClick(event.target, event.button, event.pointer_location.position)
    }
}

fn requires_add_pickable(query: Query<(), (Without<Pickable>, Without<NoEditorPicking>)>) -> bool {
    !query.is_empty()
}

fn auto_add_editor_picking_set(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    meshes_query: Query<(Entity, &Handle<Mesh>), (Without<Pickable>, Without<NoEditorPicking>)>,
) {
    for (entity, handle) in meshes_query.iter() {
        if let Some(mesh) = meshes.get(handle) {
            info!("Adding pickable to mesh {:?}", entity);
            if let PrimitiveTopology::TriangleList = mesh.primitive_topology() {
                commands.entity(entity).insert((
                    PickableBundle::default(),
                    HIGHLIGHT_TINT,
                    On::<Pointer<Click>>::send_event::<PointerClick>(),
                ));
            }
        }
    }
}

const HIGHLIGHT_TINT: Highlight<StandardMaterial> = Highlight {
    hovered: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + Color::rgba(-0.5, -0.3, 0.9, 0.8), // hovered is blue
        ..matl.to_owned()
    })),
    pressed: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + Color::rgba(-0.4, -0.4, 0.8, 0.8), // pressed is a different blue
        ..matl.to_owned()
    })),
    selected: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + Color::rgba(-0.4, 0.8, -0.4, 0.0), // selected is green
        ..matl.to_owned()
    })),
};

fn handle_events(
    mut click_events: EventReader<PointerClick>,
    mut editor: ResMut<Editor>,
    input: Res<ButtonInput<KeyCode>>,
    // egui_entity: Query<&EguiPointer>,
    mut egui_contexts: EguiContexts,
) {
    for click in click_events.read() {
        info!("Handling click event {:?} {:?}", click, editor.active());
        if !editor.active() {
            return;
        }

        if click.1 != PointerButton::Primary {
            continue;
        }

        if !editor.viewport().contains(Pos2::new(click.2.x, click.2.y))
            && egui_contexts.ctx_mut().wants_pointer_input()
        {
            continue;
        }

        // if egui_entity.get(click.0).is_ok() || egui_contexts.ctx_mut().wants_pointer_input() {
        //     continue;
        // };

        let state = editor.window_state_mut::<HierarchyWindow>().unwrap();

        let ctrl = input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
        let shift = input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let mode = SelectionMode::from_ctrl_shift(ctrl, shift);

        let entity = click.0;
        info!("Selecting mesh, found {:?}", entity);
        state
            .selected
            .select(mode, entity, |_, _| std::iter::once(entity));
    }
}
