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

#[must_use]
pub fn build_batches_with_budget(
    segments: &[TranslationSegment],
    maximum_size: usize,
    maximum_characters: usize,
) -> Vec<Vec<TranslationSegment>> {
    let maximum_size = maximum_size.max(1);
    let maximum_characters = maximum_characters.max(1);
    let mut batches: Vec<Vec<TranslationSegment>> = Vec::new();
    for segment in segments {
        let segment_characters = segment.source.chars().count();
        let can_append = batches.last().is_some_and(|batch| {
            batch.len() < maximum_size
                && batch
                    .first()
                    .is_some_and(|first| first.scene == segment.scene)
                && batch
                    .iter()
                    .map(|item| item.source.chars().count())
                    .sum::<usize>()
                    + segment_characters
                    <= maximum_characters
        });
        if can_append {
            if let Some(batch) = batches.last_mut() {
                batch.push(segment.clone());
            }
        } else {
            batches.push(vec![segment.clone()]);
        }
    }
    batches
}
