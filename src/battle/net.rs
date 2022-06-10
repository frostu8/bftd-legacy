//! A networked battle using [`backroll`].

use super::script::Scope;
use super::{Arena, State, FRAMES_PER_SECOND};

use crate::input::{sampler::Handle as InputHandle, Buffer as InputBuffer, Inputs};
use crate::render::Renderer;
use crate::Context;

use backroll::{
    command::{Command, Commands},
    P2PSession, P2PSessionBuilder, PlayerHandle, BackrollError,
};
use backroll_transport_udp::{UdpManager, UdpConnectionConfig};

use anyhow::Error;

use std::net::{ToSocketAddrs, SocketAddr};
use std::hash::{Hash, Hasher};
use std::mem::MaybeUninit;

/// A networked battle manager with a local player and a remote peer.
pub struct NetBattle {
    arena: Arena,
    session: P2PSession<NetConfig>,
    _transport: UdpManager,
    // the player at index 0 is left, index 1 is right.
    players: [Player; 2],
}

/// Config for use in initialization of a [`NetBattle`].
#[derive(Clone)]
pub enum NetPlayer {
    /// A local player with an input handle.
    Local(InputHandle),
    /// A remote player with a [`SocketAddr`] to bind to.
    Remote(SocketAddr),
}

impl NetBattle {
    /// Creates a new `NetBattle` with a given config.
    ///
    /// This does not perform any I/O and just sets up reading and writing. The
    /// [`Arena`] passed must have been synced beforehand. This struct can also
    /// be used to spectate games!
    ///
    /// # Panics
    /// Panics if more than one local player is supplied. Only give one!
    pub fn new(
        cx: &mut Context,
        arena: Arena,
        bind_addrs: impl ToSocketAddrs,
        in_players: &[NetPlayer; 2],
    ) -> Result<NetBattle, Error> {
        // initialize transport
        let transport = UdpManager::bind(cx.task_pool.clone(), bind_addrs)?;

        // initialize session
        let mut session = P2PSessionBuilder::<NetConfig>::new().with_frame_delay(0);

        let mut players: [MaybeUninit<Player>; 2] = MaybeUninit::uninit_array();
        
        for (i, player) in in_players.into_iter().enumerate() {
            match player {
                NetPlayer::Local(p) => {
                    let handle = session.add_player(backroll::Player::Local);
                    
                    players[i] = MaybeUninit::new(Player {
                        kind: PlayerKind::Local(*p),
                        handle,
                        inputs: Default::default(),
                    });
                }
                NetPlayer::Remote(addr) => {
                    let peer = transport.connect(UdpConnectionConfig::bounded(*addr, 5));
                    let handle = session.add_player(backroll::Player::Remote(peer));

                    players[i] = MaybeUninit::new(Player {
                        kind: PlayerKind::Remote,
                        handle,
                        inputs: Default::default(),
                    });
                }
            }
        }

        let session = session.start(cx.task_pool.clone())?;

        Ok(NetBattle {
            arena,
            session,
            _transport: transport,
            // SAFETY: the `in_players` array passed must be at least 2, the
            // length of the uninit array. the loop above initializes this array
            players: unsafe { MaybeUninit::array_assume_init(players) },
        })
    }

    /// Polls an update for the `NetBattle`.
    pub fn update(&mut self, cx: &mut Context) -> Result<(), Error> {
        self.handle_commands(cx, self.session.poll())?;

        'update: while cx.frame_limiter.should_update(FRAMES_PER_SECOND) {
            // only run logic if the session is synchronized
            if self.session.is_synchronized() {
                // sample input from the local player(s)
                for player in self.players.iter() {
                    if let Some(input) = player.sample_local(cx) {
                        match self.session.add_local_input(player.handle, input) {
                            Ok(()) => (),
                            Err(BackrollError::ReachedPredictionBarrier) => {
                                warn!("skipping rollback frame {}", self.arena.frame());
                                continue 'update;
                            }
                            Err(e) => return Err(e.into()),
                        };
                    }
                }

                // handle commands
                self.handle_commands(cx, self.session.advance_frame())?;
            }
        }

        Ok(())
    }

    /// Draws the battle to a graphics context.
    pub fn draw(&mut self, cx: &mut Renderer) -> Result<(), Error> {
        self.arena.draw(cx)
    }

    fn handle_commands(
        &mut self,
        cx: &mut Context,
        cmds: Commands<NetConfig>,
    ) -> Result<(), Error> {
        for cmd in cmds {
            match cmd {
                Command::AdvanceFrame(inputs) => {
                    // do game logic here!
                    for player in self.players.iter_mut() {
                        player.inputs.push(*inputs.get(player.handle).unwrap());
                    }

                    self.arena.update(
                        &cx.script,
                        &self.players[0].inputs,
                        &self.players[1].inputs,
                    )?;
                }
                Command::Save(save) => {
                    // take a snapshot
                    let snapshot = ArenaSnapshot::snapshot(&self.arena);

                    save.save(snapshot);
                }
                Command::Load(save) => {
                    // load snapshot.
                    save.load().impose(&mut self.arena);
                }
                Command::Event(ev) => match ev {
                    _ => (),
                },
            }
        }

        Ok(())
    }
}

impl Player {
    fn sample_local(&self, cx: &mut Context) -> Option<Inputs> {
        match self.kind {
            PlayerKind::Local(input) => Some(cx.input.sample(input).unwrap_or_default()),
            _ => None,
        }
    }
}

struct NetConfig;

impl backroll::Config for NetConfig {
    type State = ArenaSnapshot;
    type Input = Inputs;
}

struct Player {
    kind: PlayerKind,
    handle: PlayerHandle,
    inputs: InputBuffer,
}

enum PlayerKind {
    Local(InputHandle),
    Remote,
}

#[derive(Clone, Hash)]
struct ArenaSnapshot {
    p1: PlayerSnapshot,
    p2: PlayerSnapshot,
}

#[derive(Clone)]
struct PlayerSnapshot {
    scope: Scope<'static>,
    state: State,
}

impl ArenaSnapshot {
    /// Takes a snapshot of the arena.
    pub fn snapshot(arena: &Arena) -> ArenaSnapshot {
        ArenaSnapshot {
            p1: PlayerSnapshot::snapshot(&arena.p1),
            p2: PlayerSnapshot::snapshot(&arena.p2),
        }
    }

    /// Imposes this snapshot upon an arena.
    pub fn impose(self, arena: &mut Arena) {
        self.p1.impose(&mut arena.p1);
        self.p2.impose(&mut arena.p2);
    }
}

impl PlayerSnapshot {
    /// Takes a snapshot of the player.
    pub fn snapshot(player: &super::Player) -> PlayerSnapshot {
        PlayerSnapshot {
            scope: player.scope.clone_visible(),
            state: player.state.clone(),
        }
    }

    /// Imposes this snapshot upon a player.
    pub fn impose(self, player: &mut super::Player) {
        player.scope = self.scope;
        player.state = self.state;
    }
}

impl Hash for PlayerSnapshot {
    fn hash<H>(&self, h: &mut H)
    where
        H: Hasher,
    {
        // only hash state LOL
        self.state.hash(h);
    }
}

