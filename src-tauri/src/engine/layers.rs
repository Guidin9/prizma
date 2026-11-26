//! Layered scenes: several software effects stacked bottom to top, each limited to a set of keys.

use super::effects::{self, Inputs, Rgb, State, SwEffect};
use crate::layout::Layout;
use serde::{Deserialize, Serialize};

fn one() -> f32 {
    1.0
}

fn yes() -> bool {
    true
}

/// How a layer combines with what is below it.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Blend {
    /// covers what is below (by opacity)
    #[default]
    Normal,
    /// adds its light on top: dark parts leave the layer below untouched
    Add,
    /// keeps whichever is brighter, per channel
    Max,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Layer {
    pub effect: SwEffect,
    #[serde(default = "one")]
    pub opacity: f32,
    #[serde(default)]
    pub blend: Blend,
    /// key ids (`Key::ansi`) the layer covers; empty = the whole keyboard. Ids, not indices, so a
    /// scene survives switching between the TR and ANSI layouts.
    #[serde(default)]
    pub keys: Vec<String>,
    #[serde(default = "yes")]
    pub visible: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
pub struct Scene {
    /// bottom first
    #[serde(default)]
    pub layers: Vec<Layer>,
}

impl Scene {
    #[cfg(test)]
    pub fn single(effect: SwEffect) -> Self {
        Scene { layers: vec![Layer { effect, opacity: 1.0, blend: Blend::Normal, keys: Vec::new(), visible: true }] }
    }

    /// Effect kinds of the visible layers (decides which inputs must run).
    pub fn kinds(&self) -> impl Iterator<Item = &str> {
        self.layers.iter().filter(|l| l.visible).map(|l| l.effect.kind.as_str())
    }
}

/// Per-layer effect state, kept across frames. Indexed like `Scene::layers`.
#[derive(Default)]
pub struct SceneState {
    layers: Vec<(State, Vec<Rgb>)>,
}

/// Render every visible layer and blend them, bottom to top, into `out` (one colour per key).
pub fn render(scene: &Scene, layout: &Layout, t: f32, inp: &Inputs, st: &mut SceneState, out: &mut Vec<Rgb>) {
    let n = layout.keys.len();
    out.clear();
    out.resize(n, [0.0; 3]);
    st.layers.resize_with(scene.layers.len(), Default::default);
    for (layer, (ls, buf)) in scene.layers.iter().zip(st.layers.iter_mut()) {
        if !layer.visible || layer.opacity <= 0.0 {
            continue;
        }
        effects::render(&layer.effect, layout, t, inp, ls, buf);
        let a = layer.opacity.clamp(0.0, 1.0);
        for (i, key) in layout.keys.iter().enumerate() {
            if !layer.keys.is_empty() && !layer.keys.iter().any(|k| *k == key.ansi) {
                continue;
            }
            let (below, top) = (out[i], buf[i]);
            out[i] = match layer.blend {
                Blend::Normal => [0, 1, 2].map(|c| below[c] + (top[c] - below[c]) * a),
                Blend::Add => [0, 1, 2].map(|c| (below[c] + top[c] * a).min(255.0)),
                Blend::Max => [0, 1, 2].map(|c| below[c].max(top[c] * a)),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::effects::{AudioLevels, ScreenGrid};

    fn solid(rgb: [u8; 3]) -> SwEffect {
        SwEffect { kind: "solid".into(), speed: 1.0, brightness: 1.0, colours: vec![rgb], direction: String::new(), scale: 1.0 }
    }

    fn run(scene: &Scene) -> (Layout, Vec<Rgb>) {
        let layout = crate::layout::load(crate::k617::LAYOUTS_JSON, "tr_iso").unwrap();
        let (audio, screen) = (AudioLevels::default(), ScreenGrid::default());
        let inp = Inputs { presses: &[], audio: &audio, screen: &screen };
        let mut out = Vec::new();
        render(scene, &layout, 0.5, &inp, &mut SceneState::default(), &mut out);
        (layout, out)
    }

    fn at(layout: &Layout, out: &[Rgb], id: &str) -> [u8; 3] {
        let i = layout.keys.iter().position(|k| k.ansi == id).unwrap();
        out[i].map(|v| v.round() as u8)
    }

    #[test]
    fn masked_layer_only_touches_its_keys() {
        let mut scene = Scene::single(solid([0, 0, 255]));
        scene.layers.push(Layer { keys: vec!["W".into(), "A".into(), "S".into(), "D".into()], ..Scene::single(solid([255, 0, 0])).layers[0].clone() });
        let (layout, out) = run(&scene);
        assert_eq!(at(&layout, &out, "W"), [255, 0, 0]);
        assert_eq!(at(&layout, &out, "D"), [255, 0, 0]);
        assert_eq!(at(&layout, &out, "Q"), [0, 0, 255]);
    }

    #[test]
    fn blend_modes() {
        let mut scene = Scene::single(solid([100, 0, 0]));
        let top = |blend, opacity| Layer { blend, opacity, ..Scene::single(solid([0, 200, 0])).layers[0].clone() };
        scene.layers.push(top(Blend::Normal, 0.5));
        let (l, out) = run(&scene);
        assert_eq!(at(&l, &out, "Q"), [50, 100, 0]);
        scene.layers[1] = top(Blend::Add, 1.0);
        let (l, out) = run(&scene);
        assert_eq!(at(&l, &out, "Q"), [100, 200, 0]);
        scene.layers[1] = top(Blend::Max, 0.25);
        let (l, out) = run(&scene);
        assert_eq!(at(&l, &out, "Q"), [100, 50, 0]);
    }

    #[test]
    fn hidden_layers_are_skipped() {
        let mut scene = Scene::single(solid([10, 20, 30]));
        scene.layers.push(Layer { visible: false, ..Scene::single(solid([255, 255, 255])).layers[0].clone() });
        let (l, out) = run(&scene);
        assert_eq!(at(&l, &out, "Space"), [10, 20, 30]);
    }
}
