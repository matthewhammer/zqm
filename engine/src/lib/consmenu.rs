use serde::{Deserialize, Serialize};
use types::lang::Name;

pub type Text = String;
pub type Nat = usize; // todo -- use a bignum rep
pub type Label = Name;


#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Type {
    Prim(PrimType),
    Variant(Vec<(Label, Type)>),
    Product(Vec<(Label, Type)>),
    Option(Box<Type>),
    Vec(Box<Type>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum PrimType {
    Nat,
    Text
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum MenuChoice {
    Blank,
    Nat(Nat),
    Text(Text),
    Variant(Label, Box<MenuChoice>),
    Product(Vec<(Label, MenuChoice)>),
    Some(Box<MenuChoice>),
    None,
    Vec(Vec<MenuChoice>),
    Error(Error, Box<MenuChoice>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Error {
    TypeMismatch(Type, Tag), // found vs expected
    Blank(Tag), // found blank vs expected completed 'tag'
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Tag {
    Prim(PrimType),
    Variant,
    Product,
    Option,
    Vec,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum InitCommand {
    DefaultChoice(MenuChoice, Type)
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum AutoCommand {
    CheckType,
    CheckComplete,
    Replace(MenuChoice),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum EditCommand {
    GotoRoot,
    NextBlank,
    VecInsertBlank,
    VariantNext,
    VariantAccept,
    VariantReset,
    Choose(MenuChoice),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum MenuCtx {
    Product(LabeledChoice<Box<MenuCtx>>),
    Variant(LabeledChoice<Box<MenuCtx>>),
    Option(bool, Box<MenuCtx>),
    Vec(Vec<MenuTree>, Box<MenuCtx>, Vec<MenuTree>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct LabeledChoice<X> {
    before_choice: Vec<(Label, MenuTree)>,
    choice: Option<(Label, X)>,
    after_choice: Vec<(Label, MenuTree)>
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum MenuTree {
    Product(Vec<(Label, MenuTree)>),
    Variant(LabeledChoice<Box<MenuTree>>),
    Option(bool, Box<MenuTree>),
    Vec(Vec<MenuTree>),
    Blank,
    Nat(Nat),
    Text(Text),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Menu {
    pub ctx: MenuCtx,
    pub tree: MenuTree,
    pub choice: MenuChoice,
}


pub fn choice_of_menu(menu:&Menu) -> Result<MenuChoice, Error> {

    #[allow(unused_variables, unused_mut)]
    pub fn choice_of_ctx(ctx: &MenuCtx, local_choice:MenuChoice) -> Result<MenuChoice, Error> {
        unimplemented!()
    };    

    #[allow(unused_variables, unused_mut)]
    pub fn choice_of_tree(tree:&MenuTree) -> Result<MenuChoice, Error> {
        unimplemented!()
    };

    let choice = choice_of_tree(&menu.tree)?;
    let choice = choice_of_ctx(&menu.ctx, choice)?;
    Ok(choice)
}

