use core::fmt::Debug;
use slotmap::{SlotMap, HopSlotMap, DenseSlotMap, Key, Slottable};
use super::Storage;

unsafe impl<K, V> Storage for SlotMap<K, V>
where
    K: Key + Debug + Eq,
    V: Slottable,
{
    type Key = K;
    type Element = V;
    // Those methods clone the keys which have been fed into them — this is perfectly fine, since
    // slotmap keys are actually Copy
    fn add(&mut self, element: Self::Element) -> Self::Key {
        self.insert(element)
    }
    fn remove(&mut self, key: &Self::Key) -> Self::Element {
        self.remove(key.clone())
            .expect("the value with this key has already been removed")
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_key(capacity)
    }
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Element {
        self.get_unchecked(key.clone())
    }
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Element {
        self.get_unchecked_mut(key.clone())
    }
    fn contains_key(&self, key: &Self::Key) -> bool {
        self.contains_key(key.clone())
    }
    fn get(&self, key: &Self::Key) -> Option<&Self::Element> {
        self.get(key.clone())
    }
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Element> {
        self.get_mut(key.clone())
    }
    fn capacity(&self) -> usize {
        self.capacity()
    }
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional)
    }
    fn shrink_to_fit(&mut self) {
        // FIXME slotmaps don't have a shrink_to_fir method
    }
}

unsafe impl<K, V> Storage for HopSlotMap<K, V>
where
    K: Key + Debug + Eq,
    V: Slottable,
{
    type Key = K;
    type Element = V;
    // Those methods clone the keys which have been fed into them — this is perfectly fine, since
    // slotmap keys are actually Copy
    fn add(&mut self, element: Self::Element) -> Self::Key {
        self.insert(element)
    }
    fn remove(&mut self, key: &Self::Key) -> Self::Element {
        self.remove(key.clone())
            .expect("the value with this key has already been removed")
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_key(capacity)
    }
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Element {
        self.get_unchecked(key.clone())
    }
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Element {
        self.get_unchecked_mut(key.clone())
    }
    fn contains_key(&self, key: &Self::Key) -> bool {
        self.contains_key(key.clone())
    }
    fn get(&self, key: &Self::Key) -> Option<&Self::Element> {
        self.get(key.clone())
    }
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Element> {
        self.get_mut(key.clone())
    }
    fn capacity(&self) -> usize {
        self.capacity()
    }
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional)
    }
    fn shrink_to_fit(&mut self) {
        // FIXME slotmaps don't have a shrink_to_fir method
    }
}

unsafe impl<K, V> Storage for DenseSlotMap<K, V>
where
    K: Key + Debug + Eq,
    V: Slottable,
{
    type Key = K;
    type Element = V;
    // Those methods clone the keys which have been fed into them — this is perfectly fine, since
    // slotmap keys are actually Copy
    fn add(&mut self, element: Self::Element) -> Self::Key {
        self.insert(element)
    }
    fn remove(&mut self, key: &Self::Key) -> Self::Element {
        self.remove(key.clone())
            .expect("the value with this key has already been removed")
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_key(capacity)
    }
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Element {
        self.get_unchecked(key.clone())
    }
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Element {
        self.get_unchecked_mut(key.clone())
    }
    fn contains_key(&self, key: &Self::Key) -> bool {
        self.contains_key(key.clone())
    }
    fn get(&self, key: &Self::Key) -> Option<&Self::Element> {
        self.get(key.clone())
    }
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Element> {
        self.get_mut(key.clone())
    }
    fn capacity(&self) -> usize {
        self.capacity()
    }
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional)
    }
    fn shrink_to_fit(&mut self) {
        // FIXME slotmaps don't have a shrink_to_fir method
    }
}
