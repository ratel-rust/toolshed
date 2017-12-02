use list::List;
use map::{Map, BloomMap};
use set::{Set, BloomSet};

impl<'a, 'b, A, B> PartialEq<List<'b, B>> for List<'a, A>
where
    A: PartialEq<B>,
{
    #[inline]
    fn eq(&self, other: &List<'b, B>) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'a, 'b, KA, VA, KB, VB> PartialEq<Map<'b, KB, VB>> for Map<'a, KA, VA>
where
    (&'a KA, VA): PartialEq<(&'b KB, VB)>,
    VA: Copy,
    VB: Copy,
{
    #[inline]
    fn eq(&self, other: &Map<'b, KB, VB>) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'a, 'b, KA, VA, KB, VB> PartialEq<BloomMap<'b, KB, VB>> for BloomMap<'a, KA, VA>
where
    (&'a KA, VA): PartialEq<(&'b KB, VB)>,
    VA: Copy,
    VB: Copy,
{
    #[inline]
    fn eq(&self, other: &BloomMap<'b, KB, VB>) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'a, 'b, A, B> PartialEq<Set<'b, B>> for Set<'a, A>
where
    A: PartialEq<B>,
{
    #[inline]
    fn eq(&self, other: &Set<'b, B>) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'a, 'b, A, B> PartialEq<BloomSet<'b, B>> for BloomSet<'a, A>
where
    A: PartialEq<B>,
{
    #[inline]
    fn eq(&self, other: &BloomSet<'b, B>) -> bool {
        self.iter().eq(other.iter())
    }
}
