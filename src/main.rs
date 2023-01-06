//! Example of the visitor pattern in Rust.
//! Since Rust has neither function overloading, nor
//! inheritance, the resulting implementation is quite
//! a bit different than the equivalent C++ or Java
//! snippets.
//!
//! The motivation for this example is to provide a
//! simple demonstration of how we treat parse trees in
//! our code hardening.
//!
//! The reason this demonstration is needed is that
//! online resources on the use of this pattern in Rust
//! tend to lean toward a specific domain's use cases,
//! like Serde's method of (de)serializing,
//! or the way rustc (The Rust compiler) traverses
//! its AST.
//!
//! We are mostly inspired by the latter, but differ in
//! a few cases where we don't require the full complexity
//! of a compiler's traversal mechanisms.

/// Thing represents nodes in our parse tree.
/// Chests and Piles are non-terminal nodes that
/// may contain other nodes, while Apples and
/// Bananas are terminal nodes that contain only
/// themselves.
enum Thing {
    Chest(Chest),
    Pile(Pile),
    Apple(Apple),
    Banana(Banana),
}

struct Chest {
    upper_compartment: Vec<Thing>,
    lower_compartment: Vec<Thing>,
}

struct Pile {
    surface: Vec<Thing>,
    inside: Vec<Thing>,
    lost_forever: Vec<Thing>,
}

struct Apple;
struct Banana;

/// The Visitor trait performs the same function as a Visitor
/// abstract class in Java or C++.
///
/// Since Rust does not have inheritance, we cannot simply
/// extend the base implementation by selectively overriding
/// methods or member functions of the base class - but we can
/// get the same behavior by defining default implementations
/// for the trait methods that do nothing, as seen below.
///
/// In this use of the visitor pattern, the visitor itself
/// represents an operation we perform on our parse tree.
/// This is the reason for the associated `Value` type inside
/// the trait, and the `value` member function that retrieves
/// the end result of running the visitor operation on a collection.
///
/// To modify the internal `Value` of the visitor, all the
/// `visit_something` methods take the visitor as a mutable reference,
/// while all the visited nodes are visited immutably.
/// It is possible to also take the nodes as mutable references by
/// clever application of interior mutability, but this is generally
/// not needed, as we don't seek to destructively process our parse
/// tree.
trait Visitor {
    type Value;

    fn visit_chest(&mut self, _: &Chest) {}
    fn visit_pile(&mut self, _: &Pile) {}
    fn visit_apple(&mut self, _: &Apple) {}
    fn visit_banana(&mut self, _: &Banana) {}

    fn value(&self) -> Self::Value;
}

/// In Java and C++ it is common to define an `accept` method
/// for Visitor base classes on anything that is to be traversed.
///
/// The Rust terminology for the same concept is `walk_something`,
/// where `something` specifies what we are traversing.
/// This is the style used in rustc
/// <https://doc.rust-lang.org/beta/nightly-rustc/rustc_ast/visit/index.html>.
fn walk_things<V: Visitor>(visitor: &mut V, things: &[Thing]) {
    for thing in things {
        match thing {
            Thing::Chest(ref chest) => visitor.visit_chest(chest),
            Thing::Pile(ref pile) => visitor.visit_pile(pile),
            Thing::Apple(ref apple) => visitor.visit_apple(apple),
            Thing::Banana(ref banana) => visitor.visit_banana(banana),
        }
    }
}

/// Using only `walk_things` above, we need to manually call the
/// method for both the upper- and lower compartment of Chests.
/// If we don't require this fine grained control, we can use this
/// method instead, which walks the visitor through the entire Chest,
/// without requiring that the visitor knows anything about its
/// structure.
fn walk_chest<V: Visitor>(visitor: &mut V, chest: &Chest) {
    walk_things(visitor, &chest.upper_compartment);
    walk_things(visitor, &chest.lower_compartment);
}

/*
 * Sidenote: It is possible to use more traits and generics to fake function
 * overloading, so we only need to see a single `visit`- and `walk` method.
 *
 * See an example of this underneath, but note that this implementation is
 * not recommended, because no typing is saved (you still have to implement
 * the method for each type), and the code becomes less explicit.
 *
 * trait Walkable {
 *     fn walk<V: Visitor>(&self, visitor: &mut V);
 * }
 * impl Walk for Vec<Thing> {
 *     fn walk<V: Visitor>(&self, visitor: &mut V) {
 *         for thing in self.iter() {
 *             match thing {
 *                 Thing::Chest(ref chest) => visitor.visit_chest(chest),
 *                 Thing::Pile(ref pile) => visitor.visit_pile(pile),
 *                 Thing::Apple(ref apple) => visitor.visit_apple(apple),
 *                 Thing::Banana(ref banana) => visitor.visit_banana(banana),
 *             }
 *         }
 *     }
 * }
 *
 */

/// Any homogeneous algorithm that does not destructively visit
/// our parse tree may now be represented as _something_, implementing
/// the Visitor trait.
#[derive(Default)]
struct InventoryCounter {
    apples: usize,
    bananas: usize,
}

impl Visitor for InventoryCounter {
    type Value = String;

    fn visit_chest(&mut self, v: &Chest) {
        walk_chest(self, v);
    }

    fn visit_pile(&mut self, v: &Pile) {
        walk_things(self, &v.surface);
        walk_things(self, &v.inside);
        walk_things(self, &v.lost_forever);
    }

    fn visit_apple(&mut self, _: &Apple) {
        self.apples += 1;
    }

    fn visit_banana(&mut self, _: &Banana) {
        self.bananas += 1;
    }

    fn value(&self) -> Self::Value {
        format!("{} apples and {} bananas", self.apples, self.bananas)
    }
}

/// This visitor stops when it sees a Pile, so we do not need
/// to provide an implementation for the `visit_pile` method.
/// Also notice that this visitor returns a different data type
/// than the `InventoryCounter` visitor.
#[derive(Default)]
struct OnlyCheckChests {
    apples: usize,
    bananas: usize,
}

impl Visitor for OnlyCheckChests {
    type Value = (usize, usize);

    fn visit_chest(&mut self, v: &Chest) {
        /* If a visitor requires the granularity of traversing a
         * Chest's upper- and lower compartments, that is still
         * possible, even though we defined the `walk_chest`
         * function.
         */
        walk_things(self, &v.upper_compartment);
        walk_things(self, &v.lower_compartment);
    }

    fn visit_apple(&mut self, _: &Apple) {
        self.apples += 1;
    }

    fn visit_banana(&mut self, _: &Banana) {
        self.bananas += 1;
    }

    fn value(&self) -> Self::Value {
        (self.apples, self.bananas)
    }
}

fn main() {
    let chest = Chest {
        upper_compartment: vec![Thing::Chest(Chest {
            upper_compartment: vec![Thing::Apple(Apple)],
            lower_compartment: vec![Thing::Banana(Banana), Thing::Banana(Banana)],
        })],
        lower_compartment: vec![
            Thing::Apple(Apple),
            Thing::Pile(Pile {
                surface: vec![
                    Thing::Apple(Apple),
                    Thing::Apple(Apple),
                    Thing::Banana(Banana),
                ],
                inside: vec![Thing::Apple(Apple)],
                lost_forever: vec![Thing::Banana(Banana)],
            }),
        ],
    };

    let mut inventory = InventoryCounter::default();
    inventory.visit_chest(&chest);
    println!("Inventory count: {}", inventory.value());

    let mut only_chests = OnlyCheckChests::default();
    only_chests.visit_chest(&chest);
    let result = only_chests.value();
    println!(
        "Only top-level chests: {} apples and {} bananas",
        result.0, result.1
    );
}
