use std::{collections::BTreeMap, iter};

use super::{Fragment, FragmentId, FragmentParent, FragmentPayload, Function};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Fragments {
    /// The root fragment that indirectly points to all other fragments
    pub root: FragmentId,

    pub inner: FragmentMap,
    pub by_function: Vec<Function>,
}

impl Fragments {
    /// Find the function that contains the provided fragment
    ///
    /// Any fragment that is syntactically a part of the function body will do.
    /// This specifically includes fragments within blocks that are defined in
    /// the function.
    pub fn find_function_by_fragment_in_body(
        &self,
        fragment_id: &FragmentId,
    ) -> Option<&Function> {
        let mut fragment_id = *fragment_id;

        loop {
            let fragment = self.inner.inner.get(&fragment_id)?;

            if let FragmentPayload::Function {
                inner: function, ..
            } = &fragment.payload
            {
                return Some(function);
            }

            match fragment.parent.as_ref()? {
                FragmentParent::Fragment { id } => {
                    fragment_id = *id;
                }
                FragmentParent::Function { name } => {
                    let function = self
                        .by_function
                        .iter()
                        .find(|function| &function.name == name);
                    return function;
                }
            };
        }
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct FragmentMap {
    pub inner: BTreeMap<FragmentId, Fragment>,
}

impl FragmentMap {
    pub fn iter_from(&self, id: FragmentId) -> impl Iterator<Item = &Fragment> {
        let mut next = Some(id);

        iter::from_fn(move || {
            let id = next.take()?;
            let fragment = self.inner.get(&id)?;

            next = fragment.next();

            Some(fragment)
        })
    }
}
