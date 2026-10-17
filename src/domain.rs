pub mod persistence;

/// Display fixture, not an imported or persisted record.
#[derive(Clone, Debug)]
pub struct Record {
    pub id: String,
    pub title: &'static str,
    pub category: &'static str,
    pub content: &'static str,
}

pub const SOURCE: &str = "Fictional workshop";

/// Fixed, offline examples. No personal records or generated relevance scores.
pub fn fictional_records() -> Vec<Record> {
    const NOTES: &[(&str, &str, &str)] = &[
        (
            "Aurora greenhouse",
            "Design",
            "Fictional design note: an orbital greenhouse uses violet grow lights and a closed water loop.\n\nWorkshop checklist:\n1. Inspect the fictional irrigation map.\n2. Review the sensor layout.\n3. Compare the two light schedules.\n4. Sketch the maintenance route.\n5. Label every valve.\n6. Record the simulated temperature.\n7. Review the backup power design.\n8. Walk through a fictional outage.\n9. Check the reservoir drawing.\n10. File the workshop notes.\n\nThis is a deliberately long fictional description for testing scrolling and wrapping. Nothing here describes a real installation, individual, or operational system.",
        ),
        (
            "Glass harbor",
            "Notes",
            "Fictional harbor made from recycled glass. A solar ferry carries paper lanterns across the model bay.",
        ),
        (
            "Rust workshop",
            "Learning",
            "Fictional lesson: ownership moves a notebook into a shelf; borrowing lets a reader inspect it temporarily.",
        ),
        (
            "မြန်မာစာ workshop",
            "Learning",
            "Fictional Unicode specimen: မြန်မာစာ — café, cafe\u{301}, 東京, 🦀. Display widths differ from byte lengths.",
        ),
        (
            "Lunar library",
            "Design",
            "Fictional library with quiet reading pods, amber shelf lights and a rotating archive of invented stories.",
        ),
        (
            "Blue tram",
            "Transit",
            "Fictional transit model. The blue tram visits North Garden, Paper Bridge and Lantern Square.",
        ),
        (
            "Paper bridge",
            "Design",
            "Fictional bridge built from folded paper for a tabletop engineering class.",
        ),
        (
            "Rain collector",
            "Notes",
            "Fictional roof collects simulated rain. Compare a mesh filter with a settling basin.",
        ),
        (
            "Lantern archive",
            "Archive",
            "Fictional archive of lantern drawings: round, square, folded and star-shaped.",
        ),
        (
            "Signal garden",
            "Design",
            "Fictional public garden where light patterns represent imaginary plant growth.",
        ),
        (
            "Ceramic battery",
            "Learning",
            "Fictional energy-storage exercise; an illustration rather than construction instructions.",
        ),
        (
            "Quiet station",
            "Transit",
            "Fictional station uses clear signage and broad platforms in an accessibility study.",
        ),
        (
            "Cloud atlas",
            "Archive",
            "Fictional drawings of cloud shapes above an imaginary city.",
        ),
        (
            "Copper orchard",
            "Design",
            "Fictional metal trees form a sculpture garden with shaded reading benches.",
        ),
        (
            "Night market",
            "Notes",
            "Fictional market sells imaginary maps, paper kites and painted ceramic moons.",
        ),
        (
            "Clockwork pond",
            "Design",
            "Fictional kinetic pond sculpture rotates wooden lilies at a gentle pace.",
        ),
        (
            "Rust workshop",
            "Learning",
            "Fictional second lesson with a duplicate title: enums describe explicit states, and Result describes failure.",
        ),
        (
            "Mirror observatory",
            "Archive",
            "Fictional observatory studies reflections in a model sky; no real observations are included.",
        ),
        (
            "Bamboo terminal",
            "Design",
            "Fictional terminal design uses cyan focus borders and a quiet violet background.",
        ),
        (
            "Quote collection",
            "Learning",
            "Fictional typography sample: \"measure twice\", apostrophes aren't commands, and <tags> are plain text.",
        ),
        (
            "Moss courtyard",
            "Notes",
            "Fictional courtyard pairs a shallow pond with a shaded moss wall.",
        ),
        (
            "Solar ferry",
            "Transit",
            "Fictional ferry schedule from a tabletop city. No real departure times or passenger records.",
        ),
        (
            "Prism notebook",
            "Archive",
            "Fictional notebook of color studies: cyan, violet, amber and green.",
        ),
        (
            "End of the map",
            "Notes",
            "Fictional final fixture. This record helps verify navigation to the end of a scrolling list.",
        ),
    ];
    NOTES
        .iter()
        .enumerate()
        .map(|(index, &(title, category, content))| Record {
            id: format!("00000000-0000-4000-8000-{:012}", index + 1),
            title,
            category,
            content,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixtures_are_bounded_unique_and_explicitly_fictional() {
        let records = fictional_records();
        assert!((20..=50).contains(&records.len()));
        let ids: std::collections::HashSet<_> = records.iter().map(|r| &r.id).collect();
        assert_eq!(ids.len(), records.len());
        assert!(records.iter().all(|r| r.content.starts_with("Fictional")));
        assert_eq!(
            records.last().unwrap().id,
            fictional_records().last().unwrap().id
        );
    }
}
