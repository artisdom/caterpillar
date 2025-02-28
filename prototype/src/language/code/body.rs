use super::{Expression, Fragment, FragmentId, FragmentKind, Fragments};

#[derive(Clone, Debug, Default, Eq, PartialEq, udigest::Digestable)]
pub struct Body {
    inner: Vec<FragmentId>,
}

impl Body {
    pub fn push(
        &mut self,
        fragment: Fragment,
        fragments: &mut Fragments,
    ) -> FragmentId {
        let id = fragments.insert(fragment);
        self.inner.push(id);
        id
    }

    pub fn entry(&self) -> Option<&FragmentId> {
        self.inner.first()
    }

    pub fn ids(&self) -> impl DoubleEndedIterator<Item = &FragmentId> {
        self.inner.iter()
    }

    pub fn fragments<'r>(
        &'r self,
        fragments: &'r Fragments,
    ) -> impl Iterator<Item = &'r Fragment> {
        self.ids().map(|hash| fragments.get(hash))
    }

    pub fn expression<'r>(
        &'r self,
        fragments: &'r Fragments,
    ) -> Option<(&'r Expression, &'r Body)> {
        self.fragments(fragments).find_map(|fragment| {
            if let FragmentKind::Expression { expression } = &fragment.kind {
                Some((expression, &fragment.body))
            } else {
                None
            }
        })
    }

    pub fn replace(
        &mut self,
        to_replace: FragmentId,
        replacement: Fragment,
        fragments: &mut Fragments,
    ) -> FragmentId {
        for id in self.inner.iter_mut() {
            if *id == to_replace {
                let replacement = fragments.insert(replacement);
                *id = replacement;
                return replacement;
            }
        }

        panic!(
            "Expecting `Body::replace` to replace a fragment, but none was \
            found."
        );
    }
}
