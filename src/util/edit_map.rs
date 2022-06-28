use crate::tokenizer::Event;

/// To do: could we do without `HashMap`, so we don’t need `std`?
use std::collections::HashMap;

pub fn shift_links(events: &mut [Event], jumps: &[(usize, isize)]) {
    let map = |before| {
        let mut jump_index = 0;
        let mut jump = 0;

        while jump_index < jumps.len() {
            if jumps[jump_index].0 > before {
                break;
            }

            jump = jumps[jump_index].1;
            jump_index += 1;
        }

        #[allow(clippy::pedantic)]
        let next_i = (before as isize) + jump;
        assert!(next_i >= 0, "cannot shift before `0`");
        #[allow(clippy::pedantic)]
        let next = next_i as usize;
        next
    };

    let mut index = 0;

    while index < events.len() {
        let event = &mut events[index];
        event.previous = event.previous.map(map);
        event.next = event.next.map(map);
        index += 1;
    }
}

/// Make it easy to insert and remove things while being performant and keeping
/// links in check.
pub struct EditMap {
    consumed: bool,
    map: HashMap<usize, (usize, Vec<Event>)>,
}

impl EditMap {
    /// Create a new edit map.
    pub fn new() -> EditMap {
        EditMap {
            consumed: false,
            map: HashMap::new(),
        }
    }
    /// Create an edit: a remove and/or add at a certain place.
    pub fn add(&mut self, index: usize, mut remove: usize, mut add: Vec<Event>) {
        assert!(!self.consumed, "cannot add after consuming");

        if let Some((curr_remove, mut curr_add)) = self.map.remove(&index) {
            remove += curr_remove;
            curr_add.append(&mut add);
            add = curr_add;
        }

        self.map.insert(index, (remove, add));
    }
    /// Done, change the events.
    pub fn consume(&mut self, events: &mut [Event]) -> Vec<Event> {
        let mut indices: Vec<&usize> = self.map.keys().collect();
        let mut next_events: Vec<Event> = vec![];
        let mut start = 0;

        assert!(!self.consumed, "cannot consume after consuming");
        self.consumed = true;

        let mut index = 0;

        while index < events.len() {
            let event = &events[index];
            println!(
                "ev: {:?} {:?} {:?} {:?} {:?} {:?}",
                index,
                event.event_type,
                event.token_type,
                event.content_type,
                event.previous,
                event.next
            );
            index += 1;
        }

        indices.sort_unstable();

        let mut jumps: Vec<(usize, isize)> = vec![];
        let mut index_into_indices = 0;
        let mut shift: isize = 0;
        while index_into_indices < indices.len() {
            let index = *indices[index_into_indices];
            let edit = self.map.get(&index).unwrap();
            println!("?? {:?} {:?} {:?}", shift, edit.1.len(), edit.0);

            #[allow(clippy::pedantic)]
            let next = shift + (edit.1.len() as isize) - (edit.0 as isize);
            shift = next;
            jumps.push((index, shift));
            index_into_indices += 1;
        }

        let mut index_into_indices = 0;

        while index_into_indices < indices.len() {
            let index = *indices[index_into_indices];

            if start < index {
                let append = &mut events[start..index].to_vec();
                shift_links(append, &jumps);
                next_events.append(append);
            }

            let (remove, add) = self.map.get(&index).unwrap();

            if !add.is_empty() {
                let append = &mut add.clone();
                let mut index = 0;

                while index < append.len() {
                    let event = &mut append[index];
                    assert!(event.previous.is_none(), "to do?");
                    assert!(event.next.is_none(), "to do?");
                    index += 1;
                }

                next_events.append(append);
            }

            start = index + remove;
            index_into_indices += 1;
        }

        if start < events.len() {
            next_events.append(&mut events[start..].to_vec());
        }

        next_events
    }
}