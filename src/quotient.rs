use std::collections::HashMap;
use std::borrow::Borrow;
use std::hash::Hash;

pub trait Projection {
    type Domain;
    type Target;

    fn project(&self, x: &Self::Domain) -> Self::Target;
}

/// Quotient set
///
/// The type parameter `R` means the type of representatives.
#[derive(Clone)]
pub struct Quotient<R, T, P> {
    classes: HashMap<R, Vec<T>>,
    projection: P,
}

impl<R, T, P> Quotient<R, T, P>
where
    P: Projection<Domain=T, Target=R>,
{
    pub fn with_projection(proj: P) -> Quotient<R, T, P> {
        Self {
            classes: HashMap::new(),
            projection: proj,
        }
    }

    pub fn representatives<'a>(&'a self) -> impl Iterator<Item=&'a R> {
        self.classes.keys()
    }

    pub fn classes<'a>(&'a self) -> impl Iterator<Item=&'a Vec<T>> {
        self.classes.values()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=(&'a R, &'a Vec<T>)> {
        self.classes.iter()
    }

    pub fn len(&self) -> usize {
        self.classes.len()
    }
}

impl<R, T, P> Quotient<R, T, P>
where
    P: Projection<Domain=T, Target=R>,
    R: Eq + Hash,
{
    pub fn get<Q: ?Sized>(&self, point: &Q) -> Option<&Vec<T>>
        where
            R: Borrow<Q>,
            Q: Hash + Eq,
    {
        self.classes.get(point)
    }

    pub fn contains_representative<Q: ?Sized>(&self, point: &Q) -> bool
        where
            R: Borrow<Q>,
            Q: Hash + Eq,
    {
        self.classes.contains_key(point)
    }

    pub fn get_mut<Q: ?Sized>(&mut self, point: &Q) -> Option<&mut Vec<T>>
        where
            R: Borrow<Q>,
            Q: Hash + Eq,
    {
        self.classes.get_mut(point)
    }

    pub fn push(&mut self, item: T) {
        let repr = self.projection.project(&item);
        match self.classes.get_mut(&repr) {
            Some(class) => {
                class.push(item);
            },
            None => {
                self.classes.insert(repr, vec![item]);
            }
        }
    }

    /// Compute shallow difference
    ///
    /// This function only sees the numbers of the elements of equivalent classes.
    /// If the number of an equivalent class of `self` is greater than that of `other`,
    /// it treats that they are different.
    pub fn difference<'a, Q>(&'a self, other: &'a Quotient<R, T, Q>) -> Difference<'a, R, T, Q> {
        Difference {
            iter: self.classes.iter(),
            other: other,
        }
    }
}

pub struct Difference<'a, R, T, Q> {
    iter: std::collections::hash_map::Iter<'a, R, Vec<T>>,
    other: &'a Quotient<R, T, Q>,
}

impl<'a, R, T, Q> Iterator for Difference<'a, R, T, Q>
where
    R: Eq + Hash,
    Q: Projection<Domain=T, Target=R>,
{
    type Item = &'a R;

    fn next(&mut self) -> Option<&'a R> {
        loop {
            let (repr, class) = self.iter.next()?;
            match self.other.get(repr) {
                Some(other_class) => {
                    if class.len() > other_class.len() {
                        return Some(repr);
                    }
                },
                None => {
                    return Some(repr);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone)]
    struct Mod {
        n: usize,
    }

    impl Projection for Mod {
        type Domain = usize;
        type Target = usize;

        fn project(&self, x: &usize) -> usize {
            x % self.n
        }
    }

    #[test]
    fn get_and_contains() {
        let m10 = Mod { n: 10 };
        let mut quot: Quotient<usize, usize, _> = Quotient::with_projection(m10);
        quot.push(0);
        quot.push(1);
        quot.push(10);
        quot.push(11);
        quot.push(12);

        assert_eq!(quot.get(&0), Some(&vec![0, 10]));
        assert_eq!(quot.get(&1), Some(&vec![1, 11]));
        assert_eq!(quot.get(&2), Some(&vec![12]));
        assert_eq!(quot.get(&100), None);

        assert_eq!(quot.contains_representative(&0), true);
        assert_eq!(quot.contains_representative(&1000), false);
    }

    #[test]
    fn difference() {
        let m10 = Mod { n: 10 };
        let mut a: Quotient<usize, usize, _> = Quotient::with_projection(m10.clone());
        a.push(0);
        a.push(10);
        a.push(20);
        a.push(1);
        a.push(11);
        a.push(12);
        let mut b: Quotient<usize, usize, _> = Quotient::with_projection(m10);
        b.push(0);
        b.push(10);
        b.push(1);
        b.push(12);

        let mut diffs: Vec<usize> = a.difference(&b).cloned().collect();
        diffs.sort();

        assert_eq!(diffs, vec![0, 1]);
    }
}
