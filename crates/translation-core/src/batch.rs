#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslationSegment {
    pub id: String,
    pub scene: String,
    pub source: String,
}

impl TranslationSegment {
    #[must_use]
    pub fn new(id: impl Into<String>, scene: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            scene: scene.into(),
            source: source.into(),
        }
    }
}

#[must_use]
pub fn build_batches(
    segments: &[TranslationSegment],
    maximum_size: usize,
) -> Vec<Vec<TranslationSegment>> {
    let maximum_size = maximum_size.max(1);
    let mut scenes: Vec<(String, Vec<TranslationSegment>)> = Vec::new();
    for segment in segments {
        if let Some((_, scene_segments)) =
            scenes.iter_mut().find(|(scene, _)| scene == &segment.scene)
        {
            scene_segments.push(segment.clone());
        } else {
            scenes.push((segment.scene.clone(), vec![segment.clone()]));
        }
    }

    scenes
        .into_iter()
        .flat_map(|(_, scene_segments)| {
            scene_segments
                .chunks(maximum_size)
                .map(<[TranslationSegment]>::to_vec)
                .collect::<Vec<_>>()
        })
        .collect()
}
