use super::{Expression, Fragment, FragmentId, Fragments};

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

    pub fn ids(&self) -> impl Iterator<Item = &FragmentId> {
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
    ) -> Option<&'r Expression> {
        self.fragments(fragments).find_map(|fragment| {
            if let Fragment::Expression { expression } = fragment {
                Some(expression)
            } else {
                None
            }
        })
    }
}
