use bevy::prelude::*;

use consonant::{Consonant, ConsonantCluster, ConsonantDecoration, Digraph};
use vocal::{Vocal, VocalDecoration};

pub mod combinator;
pub mod consonant;
pub mod vocal;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Component)]
pub enum Letter {
    Vocal(Vocal),
    Consonant(ConsonantCluster),
}

impl TryFrom<&str> for Letter {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Vocal::try_from(value)
            .map(Self::Vocal)
            .or_else(|_| ConsonantCluster::try_from(value).map(Self::Consonant))
            .map_err(|_| {
                format!(
                    "Cannot assign letter to '{}' as it is not a valid letter!",
                    value
                )
            })
    }
}

impl From<ConsonantCluster> for Letter {
    fn from(value: ConsonantCluster) -> Self {
        Self::Consonant(value)
    }
}

impl From<Consonant> for Letter {
    fn from(value: Consonant) -> Self {
        ConsonantCluster::Single(value).into()
    }
}

impl From<Digraph> for Letter {
    fn from(value: Digraph) -> Self {
        ConsonantCluster::Digraph(value).into()
    }
}

impl From<Vocal> for Letter {
    fn from(value: Vocal) -> Self {
        Self::Vocal(value)
    }
}

pub trait Decorated {
    fn dots(&self) -> usize;
    fn lines(&self) -> usize;
}

impl Decorated for Letter {
    fn dots(&self) -> usize {
        match self {
            Letter::Vocal(vocal) => VocalDecoration::from(*vocal).dots(),
            Letter::Consonant(consonant) => ConsonantDecoration::from(*consonant).dots(),
        }
    }

    fn lines(&self) -> usize {
        match self {
            Letter::Vocal(vocal) => VocalDecoration::from(*vocal).lines(),
            Letter::Consonant(consonant) => ConsonantDecoration::from(*consonant).lines(),
        }
    }
}
