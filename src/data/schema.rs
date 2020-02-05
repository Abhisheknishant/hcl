use crate::app::settings::Column;
use crate::data::series::{Series, SeriesSet, Slice};

struct ColumnSchema {
    title: String,
    index: usize,
}

impl ColumnSchema {
    pub fn new(title: String, index: usize) -> ColumnSchema {
        ColumnSchema { title, index }
    }
}

///
/// Schema represents the way input data is transformed to internal format.
/// At the moment, schema has only the definition of two special fields: X and Epoch.
/// Every other field from the input will become a data series.
pub struct Schema {
    x: Option<ColumnSchema>,
    epoch: Option<ColumnSchema>,
    // titles should be also stored here.
    titles: Vec<String>,
}

impl Schema {
    fn default() -> Schema {
        Schema {
            x: None,
            epoch: None,
            titles: vec![],
        }
    }
    /// Creates new schema instance, using x/epoch configuration and
    /// titles from the input.
    pub fn new<'a, I>(x: Column, epoch: Column, titles: I) -> Schema
    where
        I: Iterator<Item = &'a str>,
    {
        let mut res = Schema::default();

        titles.zip(0..).for_each(|(t, i)| {
            if x.matches(t, i) {
                res.x = Some(ColumnSchema::new(t.to_owned(), i));
            } else if epoch.matches(t, i) {
                res.epoch = Some(ColumnSchema::new(t.to_owned(), i));
            } else {
                res.titles.push(t.to_owned());
            }
        });
        res
    }

    /// Returns a stub of SeriesSet, with correct number of
    /// empty series.
    pub fn empty_set(&self) -> SeriesSet {
        SeriesSet {
            // TODO: when do we get epoch? Only when real data arrives
            epoch: None,
            x: match self.x {
                Some(ref x) => Some((x.title.clone(), vec![])),
                _ => None,
            },
            y: self.titles.iter().map(|t| Series::with_title(t)).collect(),
        }
    }

    /// Formats a row of input data as a slice.
    /// Slice can be appended to a SeriesSet.
    pub fn slice<'a, I>(&self, input: I) -> Slice
    where
        I: Iterator<Item = &'a str>,
    {
        let mut res = Slice::default();
        input
            .enumerate()
            .for_each(|(i, v)| match (&self.x, &self.epoch) {
                (Some(x), _) if x.index == i => res.x = Some(v.to_owned()),
                (_, Some(e)) if e.index == i => res.epoch = Some(v.to_owned()),
                _ => res.y.push(v.trim().parse::<f64>().unwrap_or(std::f64::NAN)),
            });
        res
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema() {
        let schema = Schema::new(
            Column::None,
            Column::None,
            vec!["a", "b", "c"].iter().map(|s| *s),
        );
        let s = schema.empty_set();
        assert_eq!(s.epoch, None);
        assert_eq!(s.x, None);
        assert_eq!(s.y.len(), 3);
        assert_eq!(s.y[0].title, "a");
        assert_eq!(s.y[1].title, "b");
        assert_eq!(s.y[2].title, "c");

        let slice = schema.slice(vec!["1", "2", "3"].iter().map(|s| *s));
        assert_eq!(slice.epoch, None);
        assert_eq!(slice.x, None);
        assert_eq!(slice.y, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_x_epoch() {
        let schema = Schema::new(
            Column::Index(0),
            Column::Title("b".to_owned()),
            vec!["a", "b", "c"].iter().map(|s| *s),
        );
        let s = schema.empty_set();
        assert_eq!(s.epoch, None);
        assert_eq!(s.x, Some(("a".to_owned(), vec![])));
        assert_eq!(s.y.len(), 1);
        assert_eq!(s.y[0].title, "c");

        let slice = schema.slice(vec!["1", "2", "3"].iter().map(|s| *s));
        assert_eq!(slice.epoch, Some("2".to_owned()));
        assert_eq!(slice.x, Some("1".to_owned()));
        assert_eq!(slice.y, vec![3.0]);
    }
}
