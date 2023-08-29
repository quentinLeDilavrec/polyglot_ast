use std::fs::File;
use std::io::Read;

use tree_sitter::Node;

use crate::{tree_sitter_utils::TreeSitterCST, Language, PolyglotTree};

//use super::EvalSource;

use super::{
    AnaError, BuildingContext, PolyglotBuilding, PolyglotDef, PolyglotKind, PolyglotUse,
    StuffPerLanguage,
};

#[derive(Debug, PartialEq, Eq)]
pub enum UnSolvedPolyglotUse<Node> {
    // partially solved
    EvalContext { name: Node },
    // partially solved
    EvalBuilder { name: Node },
    // can be evaluated
    EvalSource { source: Node, lang: Language },
    // can be evaluated if referenced file can be evaluated
    Eval { path: Node, lang: Language },
    // can be evaluated if referenced file can be evaluated
    Import { path: Node, lang: Language },
}
impl<Ref> UnSolvedPolyglotUse<Ref> {
    pub fn get_kind(&self) -> PolyglotKind {
        match self {
            UnSolvedPolyglotUse::EvalContext { .. } => PolyglotKind::Eval,
            UnSolvedPolyglotUse::EvalBuilder { .. } => PolyglotKind::Eval,
            UnSolvedPolyglotUse::EvalSource { .. } => PolyglotKind::Eval,
            UnSolvedPolyglotUse::Eval { .. } => PolyglotKind::Eval,
            UnSolvedPolyglotUse::Import { .. } => PolyglotKind::Import,
        }
    }
}

#[derive(Debug)]
struct JavaBuilder<'tree, 'text> {
    payload: TreeSitterCST<'tree, 'text>,
    // temporary stuff
}
impl<'tree, 'text> super::PolyglotBuilding for JavaBuilder<'tree, 'text> {
    type Node = tree_sitter::Node<'tree>;
    type Ctx = TreeSitterCST<'tree, 'text>;

    fn init(payload: TreeSitterCST<'tree, 'text>) -> Self {
        Self { payload }
    }

    fn compute(self) -> PolyglotTree {
        todo!();
        // return PolyglotTree {
        //     tree: self.payload.cst,
        //     code: self.payload.code,
        //     working_dir: self.payload.working_dir,
        //     language: crate::Language::Java,
        //     node_to_subtrees_map: HashMap::new(),
        // };
    }
}
impl<'tree, 'text> JavaBuilder<'tree, 'text> {
    pub fn node_to_code(&self, node: &tree_sitter::Node<'tree>) -> &'text str {
        &self.payload.node_to_code(node)
    }
}

trait Visit {
    fn visit(&self, node: &PolyglotTree) -> Vec<Node>;
    fn display(&self, node: &PolyglotTree);
}

struct PreOrder<'tree> {
    cursor: tree_sitter::TreeCursor<'tree>,
    state: VisitState,
}
#[derive(PartialEq, Eq)]
enum VisitState {
    Down,
    Next,
    Up,
}

impl<'tree> Iterator for PreOrder<'tree> {
    type Item = Node<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == VisitState::Down {
            if self.cursor.goto_first_child() {
                self.state = VisitState::Down;
            } else {
                self.state = VisitState::Next;
                return self.next();
            }
        } else if self.state == VisitState::Next {
            if self.cursor.goto_next_sibling() {
                self.state = VisitState::Down;
            } else {
                self.state = VisitState::Up;
                return self.next();
            }
        } else if self.state == VisitState::Up {
            if self.cursor.goto_parent() {
                self.state = VisitState::Next;
                return self.next(); // TODO caution, might stack overflow
            } else {
                // finish
                //println!("ICI NEXT FAIT CRASH POLYGLOT USE");
                return None;
            }
        }

        Some(self.cursor.node())
    }
}

impl<'tree> PreOrder<'tree> {
    pub fn new(tree: &'tree tree_sitter::Tree) -> Self {
        let cursor = tree.walk();
        let state = VisitState::Down;
        Self { cursor, state }
    }
    fn node(&self) -> Node<'tree> {
        self.cursor.node()
    }

    //the visit function must use the next function to go to the next node
    fn visit(&mut self, cst: TreeSitterCST) -> Vec<Node> {
        println!("PASSAGE DANS VISIT");
        let mut nodes = Vec::new();

        while let Some(node) = self.next() {
            dbg!(node);
            nodes.push(node);
        }
        println!("FIN WHILE");

        return nodes;
    }

    fn display(&self, node: &PolyglotTree) {
        dbg!("{}", node);
    }
}

impl<'tree, 'text> StuffPerLanguage for JavaBuilder<'tree, 'text> {
    type UnsolvedUse = UnSolvedPolyglotUse<Self::Node>;
    fn find_polyglot_uses(&self) -> Vec<Self::UnsolvedUse> {
        println!("PASSAGE DANS FIND POLYGLOT USES");
        let mut uses = Vec::new();
        let tree = self.payload.cst;

        //let mut stack = vec![tree.root_node()];

        for node in PreOrder::new(tree) {
            if let Some(us) = self.try_compute_polyglot_use(&node) {
                uses.push(us.unwrap());
            }
            // dbg!(node);
            // dbg!(node.kind());
            // dbg!(node.to_sexp());
            // dbg!(node.child_count());
            // //dbg!(node.child(0));
            // if node.kind().eq("import_declaration") {
            //     let r#use = UnSolvedPolyglotUse::Import {
            //         path: self
            //             .payload
            //             .node_to_code(&node.child(1).unwrap().child(0).unwrap())
            //             .to_string(),
            //         lang: crate::Language::Java,
            //     };
            //     println!("DEBUG");
            //     dbg!(&r#use);
            //     eprintln!("{:?}", uses);
            //     uses.push(r#use);
            // } else if node.kind().eq("method_invocation") {
            //     let r#use = UnSolvedPolyglotUse::Eval {
            //         path: "??".to_string(),
            //         lang: crate::Language::Java,
            //     };
            //     dbg!(&r#use);
            //     uses.push(r#use);
            // //anciennement local_variable_declaration
            // } else if node.kind().eq(".") {
            //     println!("PASSAGE DANS .");
            //     let r#use = UnSolvedPolyglotUse::EvalBuilder {
            //         name: self
            //             .payload
            //             .node_to_code(&node.child(1).unwrap().child(0).unwrap())
            //             .to_string(),
            //     };
            //     dbg!(&r#use);
            //     uses.push(r#use);
            // } else if node.kind().eq("identifier") {
            //     println!("PASSAGE DANS IDENTIFIER");
            //     // dbg!(self.payload.node_to_code(&node.child(1).unwrap().child(0).unwrap()));
            //     // dbg!(self
            //     //     .payload
            //     //     .node_to_code(&node.child(1).unwrap().child(0).unwrap())
            //     //     .to_string());
            //     let r#use = UnSolvedPolyglotUse::EvalSource {
            //         source: self.payload.node_to_code(&node).to_string(),
            //         lang: crate::Language::Java,
            //     };
            //     println!("FIN D'INSTANCIATION DE r#use");
            //     dbg!(&r#use);
            //     uses.push(r#use);
            // } else {
            //     println!("autre")
            // }
            //node.children()
        }
        println!("FIN WHILE FIND POLYGLOT USES");
        return dbg!(uses);
    }

    fn find_polyglot_exports(&self) -> Vec<super::PolyglotDef> {
        let mut exports = Vec::new();
        let tree = self.payload.cst;
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            if self.payload.node_to_code(&tree.root_node()) == "??" {
                let r#use = PolyglotDef::ExportValue {
                    name: self
                        .payload
                        .node_to_code(&tree.root_node().child(1).unwrap())
                        .to_string(),
                    value: self
                        .payload
                        .node_to_code(&tree.root_node().child(3).unwrap())
                        .to_string(),
                };
                exports.push(r#use);
            }
        }
        return exports;
    }

    fn try_compute_polyglot_use(
        &self,
        node: &Self::Node,
    ) -> Option<Result<Self::UnsolvedUse, AnaError>> {
        let call = self.get_polyglot_call(node)?;
        let s = self.payload.node_to_code(&call);
        if s == "eval" {
            let q = tree_sitter::Query::new(
                tree_sitter_java::language(),
                r#"(method_invocation 
    name: (identifier) @name
    (#match? @name "eval")
    arguments: [
        (argument_list
            (method_invocation 
                object: (identifier) @indirect_build
                name: (identifier) @build
                (#match? @build "build")
            )
        )
        (argument_list
            (identifier) @indirect
        )
        (argument_list
            (expression) @lang
            (expression) @code
        )
    ]
)"#,
            )
            .unwrap();
            let q_c = &mut tree_sitter::QueryCursor::new();
            let mut q_res = q_c.matches(&q, node.clone(), &self.payload);
            if let Some(m) = q_res.next() {
                assert_eq!(self.payload.node_to_code(&m.captures[0].node), "eval");
                match q.capture_names()[m.captures[1].index as usize].as_str() {
                    "indirect" => {
                        dbg!(m.captures[1].node.to_sexp());
                        let indirect = self.payload.node_to_code(&m.captures[1].node);
                        dbg!(indirect);
                        todo!()
                    }
                    "indirect_build" => {
                        let indirect_build = &m.captures[1].node;
                        dbg!(self.payload.node_to_code(indirect_build));
                        return Some(Ok(UnSolvedPolyglotUse::EvalBuilder {
                            name: indirect_build.clone(),
                        }));
                    }
                    "lang" => {
                        let lang = self.payload.node_to_code(&m.captures[1].node);
                        let code = &m.captures[2].node;
                        dbg!(lang, self.payload.node_to_code(code));
                        return Some(Ok(UnSolvedPolyglotUse::EvalSource {
                            source: code.clone(),
                            lang: crate::util::language_string_to_enum(&lang).unwrap(),
                        }));
                    }
                    x => {
                        dbg!(x);
                    }
                }
            }
            todo!();
            // let arg_list = node.child(3).unwrap();
            // dbg!(self.payload.node_to_code(&arg_list));
            // dbg!(arg_list.to_sexp());
            // let r = if arg_list.child_count() == 3 {
            //     UnSolvedPolyglotUse::EvalContext {
            //         name: self
            //             .payload
            //             .node_to_code(&arg_list.child(1).unwrap())
            //             .to_string(),
            //     }
            // } else {
            //     let lang: &str = self
            //         .payload
            //         .node_to_code(&node.child(3).unwrap().child(1).unwrap());
            //     let lang = crate::util::strip_quotes(lang);
            //     dbg!(&lang);
            //     UnSolvedPolyglotUse::EvalSource {
            //         //source: "".to_string(),
            //         source: self
            //             .payload
            //             .node_to_code(&node.child(3).unwrap().child(3).unwrap())
            //             .to_string(),
            //         lang: crate::util::language_string_to_enum(&lang).unwrap(),
            //         //TODO : fix this
            //         //lang: crate::Language::Python,
            //     }
            // };
            // Some(Ok(r))
        } else if s == "getMember" {
            let r = UnSolvedPolyglotUse::Import {
                path: todo!(),
                lang: todo!(),
            };
            Some(Ok(r))
        } else {
            None
        }
    }

    fn try_compute_polyglot_def(
        &self,
        node: &Self::Node,
    ) -> Option<Result<super::PolyglotDef, AnaError>> {
        let call = self.get_polyglot_call(node)?;
        let s = self.payload.node_to_code(node);
        if s == "putMember" {
            let r = PolyglotDef::ExportValue {
                name: todo!(),
                value: todo!(),
            };
            Some(Ok(r))
        } else {
            None
        }
    }
}

impl<'tree, 'text> JavaBuilder<'tree, 'text> {
    fn get_polyglot_call<'n>(&self, node: &tree_sitter::Node<'n>) -> Option<tree_sitter::Node<'n>> {
        if node.kind().ne("method_invocation") {
            return None;
        }
        let child = node.child(2)?;
        if child.kind().eq("identifier") {
            return Some(child);
        }
        None
    }

    fn compute_polyglot_use(
        &self,
        call_expr: &<JavaBuilder<'tree, 'text> as PolyglotBuilding>::Node,
    ) -> Option<UnSolvedPolyglotUse<use_solver::Node<'tree>>> {
        // Java uses positional arguments, so they will always be accessible with the same route.

        //if node.child(3) has 1 child => code, if node.child(3) has 2 children: 1st => language, 2nd => code
        let parameters = &call_expr.child(3)?;
        let parameter_count = parameters.child_count();

        if parameter_count == 1 {
            // NOTE match something like:
            // context.eval(source);
            // where context is a org.graalvm.polyglot.Context
            // where source is a org.graalvm.polyglot.Source
            let parameter = parameters.child(1)?;

            let t = parameter.kind();
            assert_eq!(t, "identifer");

            let name = parameter;
            // let name = self.payload.node_to_code(&parameter);
            // let name = name.to_string();

            Some(UnSolvedPolyglotUse::EvalContext { name })

            // dbg!(name);
            // let a = {
            //     tree_sitter::Query::new(tree_sitter_java::language(), r#"
            //     (local_variable_declaration
            //         type: "Source" | "org.graalvm.polyglot.Source" @variable.type
            //         declarator: (variable_declarator
            //             name: (identifier @variable.name)
            //             value: (* @variable.value)))

            //     (local_variable_declaration  (arguments (identifier)))
            //     "#);
            //     todo!("the rest")
            // };

            // if let Ok(a) = a {
            //     return a;
            // }

            // let b = {
            //     todo!("with spoon")
            // };

            // // etc

            // todo!("use some robust static analysis, ie use existing lsp for Java, or Spoon, or Jdt, or tree-sitter query")
        } else if parameter_count == 5 {
            // NOTE match something like:
            // context.eval("python", "print(42)");
            // or:
            // context.eval("python", source);
            // where context is a org.graalvm.polyglot.Context
            // where source is a string or a file (or anything that can be turned into a string ?)

            //getting language and code
            let language = parameters.child(1)?;
            let code = parameters.child(3)?;
            use crate::util;
            let s = util::strip_quotes(self.payload.node_to_code(&language));

            let new_lang = match util::language_string_to_enum(&s) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Could not convert argument {s} to language due to error: {e}",);
                    return None;
                }
            };
            let new_code = util::strip_quotes(self.payload.node_to_code(&code));
            println!("{}", new_code);
            todo!()
        } else {
            None
        }
    }
}

mod use_solver {
    use std::path::PathBuf;
    #[derive(Debug, PartialEq, Eq)]
    pub struct Reference<'tree>(pub Node<'tree>);
    pub(crate) enum TwoTransitions<T0, T1> {
        T0(T0),
        T1(T1),
    }
    pub(crate) enum ThreeTransitions<T0, T1, T2> {
        T0(T0),
        T1(T1),
        T2(T2),
    }
    #[derive(Debug)]
    pub struct SolvingError;
    #[derive(Debug)]
    pub struct NoSource<S> {
        pub(crate) content: S,
    }
    impl<'tree> NoSource<Reference<'tree>> {
        pub(crate) fn solve(
            self,
            refanalysis: &'tree impl ReferenceAnalysis<Reff = Reference<'tree>>,
        ) -> Result<NoSource<Node<'tree>>, SolvingError> {
            let source = refanalysis.solve(&self.content)?;
            Ok(NoSource { content: source })
        }
    }
    impl<'tree> NoSource<Node<'tree>> {
        pub(crate) fn solve(
            self,
        ) -> Result<
            ThreeTransitions<
                NoSource<Reference<'tree>>,
                NoBuilder<Reference<'tree>>,
                PolygloteSource<Node<'tree>, Node<'tree>>,
            >,
            SolvingError,
        > {
            dbg!(self.content.to_sexp());
            todo!()
        }
    }
    #[derive(Debug)]
    pub struct NoBuilder<S> {
        pub(crate) content: S,
    }
    impl<'tree> NoBuilder<Reference<'tree>> {
        pub(crate) fn solve(
            self,
            refanalysis: &'tree impl ReferenceAnalysis<Reff = Reference<'tree>>,
        ) -> Result<NoBuilder<Node<'tree>>, SolvingError> {
            let source = refanalysis.solve(&self.content)?;
            Ok(NoBuilder { content: source })
        }
    }
    impl<'tree> NoBuilder<Node<'tree>> {
        pub(crate) fn solve(
            self,
        ) -> Result<
            TwoTransitions<NoBuilder<Reference<'tree>>, PolygloteSource<Node<'tree>, Node<'tree>>>,
            SolvingError,
        > {
            dbg!(self.content.to_sexp());
            let q = tree_sitter::Query::new(
                tree_sitter_java::language(),
                r#"(local_variable_declaration
    type: (type_identifier)  @Builder
    (#match? @Builder "Builder")
    declarator: (variable_declarator 
        name: (identifier) 
        value: [
            (identifier) @indirect_build
            (method_invocation
                object: (identifier) @Source
                (#match? @Source "Source")
                name: (identifier) @newBuilder
                (#match? @newBuilder "newBuilder")
                arguments: [
                    (argument_list
                        (string_literal) @lang0
                        (string_literal) @code0
                    )
                    (argument_list
                        (identifier) @lang1
                        (string_literal) @code1
                    )
                    (argument_list
                        (string_literal) @lang2
                        (identifier) @code2
                    )
                    (argument_list
                        (identifier) @lang3
                        (identifier) @code3
                    )
                    (argument_list
                        (string_literal) @lang4
                        (object_creation_expression
                            arguments: (argument_list
                                (string_literal) @code4
                            )
                        )
                    )
                    (argument_list
                        (identifier) @lang5
                        (object_creation_expression
                            arguments: (argument_list
                                (string_literal) @code5
                            )
                        )
                    )
                ]
            )
        ]
    )
)"#,
            )
            .unwrap();
        // TODO check qualified name of Builder
            let q_c = &mut tree_sitter::QueryCursor::new();
            // let mut q_res = q_c.matches(&q, node.clone(), &self.payload);
            // if let Some(m) = q_res.next() {
            //     assert_eq!(self.payload.node_to_code(&m.captures[0].node), "eval");
            //     match q.capture_names()[m.captures[1].index as usize].as_str() {}
            // } else {
            //     todo!()
            // }
            todo!()
        }
    }

    pub(crate) struct PolygloteSource<L, C> {
        language: L,
        code: C,
    }
    type CodeElement = (usize, usize);
    pub(crate) type Node<'tree> = tree_sitter::Node<'tree>;

    // #[cfg_attr(test, mockall::automock(type Reff=u8;))]
    pub trait ReferenceAnalysis {
        type Reff;
        fn solve<'tree>(&'tree self, reference: &Self::Reff) -> Result<Node<'tree>, SolvingError>;
    }
    impl<'tree, C> PolygloteSource<Reference<'tree>, C> {
        fn solve(
            self,
            refanalysis: &'tree impl ReferenceAnalysis<Reff = Reference<'tree>>,
        ) -> Result<PolygloteSource<Node<'tree>, C>, SolvingError> {
            let language = refanalysis.solve(&self.language)?;
            Ok(PolygloteSource {
                language,
                code: self.code,
            })
        }
    }
    impl<'tree, C> PolygloteSource<Node<'tree>, C> {
        fn solve(
            self,
        ) -> Result<
            TwoTransitions<
                PolygloteSource<Reference<'tree>, C>,
                PolygloteSource<crate::Language, C>,
            >,
            SolvingError,
        > {
            todo!()
        }
    }
    impl<'tree> PolygloteSource<crate::Language, Reference<'tree>> {
        fn solve(
            self,
            refanalysis: &'tree impl ReferenceAnalysis<Reff = Reference<'tree>>,
        ) -> Result<PolygloteSource<crate::Language, Node<'tree>>, SolvingError> {
            let code = refanalysis.solve(&self.code)?;
            Ok(PolygloteSource {
                language: self.language,
                code,
            })
        }
    }
    impl<'tree> PolygloteSource<crate::Language, Node<'tree>> {
        fn solve(
            self,
        ) -> Result<
            PolygloteSource<
                crate::Language,
                ThreeTransitions<
                    PolygloteSource<crate::Language, Reference<'tree>>,
                    PolygloteSource<crate::Language, PathBuf>,
                    PolygloteSource<crate::Language, String>,
                >,
            >,
            SolvingError,
        > {
            todo!()
        }
    }
    impl<C> PolygloteSource<crate::Language, C> {
        fn lang(&self) -> &crate::Language {
            &self.language
        }
    }
    impl<L> PolygloteSource<L, PathBuf> {
        fn code(&self) -> &PathBuf {
            &self.code
        }
    }
    impl<L> PolygloteSource<L, String> {
        fn code(&self) -> &String {
            &self.code
        }
    }
}

#[cfg(test)]
mod test {
    #[cfg(test)]
    use mockall::*;
    use std::{collections::HashMap, fmt::Display};

    use crate::{
        building::{
            java::{PreOrder, UnSolvedPolyglotUse},
            BuildingContext, PolyglotBuilding, PolygloteTreeHandle, StuffPerLanguage,
        },
        tree_sitter_utils::TreeSitterCST,
        PolyglotTree,
    };

    use super::JavaBuilder;

    fn main_wrap(main_content: impl Display) -> String {
        format!(
            "{}{}{}",
            r#"import java.io.File;

        import javax.naming.Context;
        import javax.xml.transform.Source;
        
        import org.graalvm.polyglot.Context;
        import org.graalvm.polyglot.Value;
        
        public class JavaTest2 {
            public static void main(String[] args) {"#,
            main_content.to_string(),
            r#"}}"#
        )
    }

    #[test]
    fn mock_test() {
        #[cfg_attr(test, automock)]
        trait MyTrait {
            fn foo(&self, x: u32) -> u32;
        }
        use mockall::predicate;
        fn call_with_four(x: &MockMyTrait) -> u32 {
            x.foo(4)
        }
        let mut mock = MockMyTrait::new();
        mock.expect_foo()
            .with(mockall::predicate::eq(4))
            .times(1)
            .returning(|x| x + 1);
        assert_eq!(5, call_with_four(&mock));
    }

    #[test]
    fn test_preorder_implem() {
        let main_content = r#"
        Context cx = Context.create();
        cx.eval("python", "print('hello')");
        "#;
        println!("TEST PREORDER IMPLEM");
        let file_content = main_wrap(main_content);
        let tree = crate::tree_sitter_utils::parse(&file_content);
        let cst = crate::tree_sitter_utils::into(tree.as_ref(), &file_content);
        let builder = &JavaBuilder::init(cst);
        let tree = tree.as_ref().unwrap();
        let mut pre_order = PreOrder::new(tree);
        assert_eq!(pre_order.node().kind(), "program");
        assert_eq!(
            pre_order.next().map(|n| n.kind()),
            Some("import_declaration")
        );
    }

    #[test]
    fn test_visit_function() {
        let main_content = r#"
        Context cx = Context.create();
        cx.eval("python", "print('hello')");
        "#;
        println!("TEST VISIT FUNCTION");
        let file_content = main_wrap(main_content);
        let tree = crate::tree_sitter_utils::parse(&file_content);
        let mut cst = crate::tree_sitter_utils::into(tree.as_ref(), &file_content);
        let tree = tree.as_ref().unwrap();
        let mut pre_order = PreOrder::new(tree);

        let nodes = PreOrder::visit(&mut pre_order, cst);
        assert_eq!(nodes.len(), 111);
    }

    #[test]
    fn test_polyglot_use() {
        let main_content = r#"
        Context cx = Context.create();

        Builder builder = Source.newBuilder("python", new File("TestSamples/pyprint.py"));
        cx.eval(builder.build());
        "#;
        println!("TEST POLYGLOT USE");
        let file_content = main_wrap(main_content);
        let tree = crate::tree_sitter_utils::parse(&file_content);
        let cst = crate::tree_sitter_utils::into(tree.as_ref(), &file_content);
        let builder = &JavaBuilder::init(cst);
        //dbg!(builder);

        let tree = tree.as_ref().unwrap();
        dbg!(tree.root_node().to_sexp());
        let class = tree.root_node().child(5).unwrap();
        dbg!(class.to_sexp());
        let meth = class.child(3).unwrap().child(1).unwrap();
        dbg!(meth.to_sexp());
        let poly_eval = meth.child(4).unwrap().child(3).unwrap().child(0).unwrap();
        dbg!(builder.node_to_code(&poly_eval));
        let r#use = builder.try_compute_polyglot_use(&poly_eval);
        dbg!(&r#use);
        assert_eq!(
            r#use,
            Some(Ok(UnSolvedPolyglotUse::EvalBuilder {
                name: poly_eval
                    .child(3)
                    .unwrap()
                    .child(1)
                    .unwrap()
                    .child(0)
                    .unwrap()
            },),)
        );
        struct MockedRefAna<'tree> {
            builder: &'tree JavaBuilder<'tree, 'tree>,
        };
        impl<'tree> super::use_solver::ReferenceAnalysis for MockedRefAna<'tree> {
            type Reff = super::use_solver::Reference<'tree>;
            fn solve<'tr>(
                &'tr self,
                reference: &Self::Reff,
            ) -> Result<super::use_solver::Node<'tr>, super::use_solver::SolvingError> {
                dbg!(&reference.0);
                if reference.0.start_position() == tree_sitter::Point::new(13, 16)
                    && reference.0.end_position() == tree_sitter::Point::new(13, 23)
                {
                    let tree = self.builder.payload.cst;
                    dbg!(tree.root_node().to_sexp());
                    let class = tree.root_node().child(5).unwrap();
                    dbg!(class.to_sexp());
                    let meth = class.child(3).unwrap().child(1).unwrap();
                    dbg!(meth.to_sexp());
                    let build = meth.child(4).unwrap().child(2).unwrap();
                    return Ok(build);
                } else {
                    panic!()
                }
            }
        }

        let r#use = r#use.unwrap().unwrap();

        if let UnSolvedPolyglotUse::EvalBuilder { name } = r#use {
            let r#use = super::use_solver::NoBuilder {
                content: super::use_solver::Reference(name.clone()),
            };
            let mocked_ref_ana = MockedRefAna { builder };
            let r#use = r#use.solve(&mocked_ref_ana);
            dbg!(&r#use);
            let r#use = r#use.unwrap();
            match r#use.solve().unwrap() {
                super::use_solver::TwoTransitions::T0(_) => panic!(),
                super::use_solver::TwoTransitions::T1(_) => todo!(),
            }
        } else {
            panic!()
        }

        todo!();

        let extraction = builder.find_polyglot_uses();
        dbg!(extraction);

        //extraction into find_polyglot_uses
        // let tree = tree.as_ref().unwrap();
        // println!("TEST POLYGLOT USE");
        // dbg!(tree.root_node().to_sexp());
        // let extraction = JavaBuilder::find_polyglot_uses(builder);
        // dbg!(extraction);
    }

    #[test]
    fn direct() {
        let main_content = r#"
        Context cx = Context.create();
        cx.eval("python", "print('hello')");
        "#;
        let file_content = main_wrap(main_content);
        let tree = crate::tree_sitter_utils::parse(&file_content);
        let cst = crate::tree_sitter_utils::into(tree.as_ref(), &file_content);
        let builder = &JavaBuilder::init(cst);

        // TODO extract into find_polyglot_uses
        let tree = tree.as_ref().unwrap();
        println!("TEST DIRECT");
        dbg!(tree.root_node().to_sexp());
        let class = tree.root_node().child(5).unwrap();
        dbg!(class.to_sexp());
        let meth = class.child(3).unwrap().child(1).unwrap();
        dbg!(meth.to_sexp());
        let poly_eval = meth.child(4).unwrap().child(2).unwrap().child(0).unwrap();
        //dbg!(poly_eval);
        dbg!(poly_eval.to_sexp());
        let r#use = builder.try_compute_polyglot_use(&poly_eval);
        //dbg!(r#use);
        assert_eq!(
            r#use,
            Some(Ok(UnSolvedPolyglotUse::EvalSource {
                source: poly_eval,
                lang: crate::Language::Python,
            },),)
        );
    }

    #[test]
    fn direct2() {
        let main_content = r#"
        Context cx = Context.create();
        context.eval(Source.newBuilder("python", new File("TestSamples/pyprint.py")).build());
        "#;
        let file_content = main_wrap(main_content);
        dbg!(&file_content);
        let tree = crate::tree_sitter_utils::parse(&file_content);
        dbg!(&tree);
        let cst = crate::tree_sitter_utils::into(tree.as_ref(), &file_content);
        dbg!(&cst);
        let builder = &JavaBuilder::init(cst);
        dbg!(&builder);

        // TODO extract into find_polyglot_uses
        let tree = tree.as_ref().unwrap();
        let mut pre_order = PreOrder::new(tree);

        // dbg!(pre_order.node().kind());
        // dbg!(pre_order.next().map(|n| n.kind()));
        // dbg!(pre_order.next());
        // dbg!(pre_order.next());
        // dbg!(pre_order.next());
        // dbg!(pre_order.next());
        // dbg!(pre_order.next());

        println!("DIRECT2");
        dbg!(tree.root_node().to_sexp());
        let class = tree.root_node().child(5).unwrap();
        dbg!(class.to_sexp());
        let meth = class.child(3).unwrap().child(1).unwrap();
        dbg!(meth.to_sexp());
        //let poly_eval = meth.child(4).unwrap().child(2).unwrap().child(0).unwrap().child(3).unwrap().child(1).unwrap().child(0).unwrap();
        let poly_eval = meth.child(4).unwrap().child(2).unwrap().child(0).unwrap();
        dbg!(poly_eval.child_count());
        // dbg!(poly_eval.child(0).unwrap().to_sexp());
        // dbg!(poly_eval.child(1).unwrap().to_sexp());
        dbg!(poly_eval.to_sexp());
        let r#use = builder.try_compute_polyglot_use(&poly_eval);
        dbg!(r#use);
        // assert_eq!(r#use, Some(
        //     Ok(
        //         UnSolvedPolyglotUse::EvalSource {
        //             source: "".to_string(),
        //             lang: crate::Language::Python,
        //         },
        //     )
        // ));
    }
    #[test]
    fn indirect() {
        let main_content = r#"
        Context cx = Context.create();

        Builder builder = Source.newBuilder("python", new File("TestSamples/pyprint.py"));
        context.eval(builder.build());
        "#;
        let file_content = main_wrap(main_content);
        let tree = crate::tree_sitter_utils::parse(&file_content);
        let cst = crate::tree_sitter_utils::into(tree.as_ref(), &file_content);
        let builder = &JavaBuilder::init(cst);

        // TODO extract into find_polyglot_uses
        println!("INDIRECT");
        let tree = tree.as_ref().unwrap();
        dbg!(tree.root_node().to_sexp());
        let class = tree.root_node().child(5).unwrap();
        let meth = class.child(3).unwrap().child(1).unwrap();
        let meth_body = &meth.child(4).unwrap();
        dbg!(meth_body.to_sexp());
        let poly_eval = meth_body.child(2).unwrap().child(0).unwrap();
        dbg!(poly_eval.to_sexp());
        let r#use = builder.try_compute_polyglot_use(&poly_eval);
        assert_eq!(r#use, None);
    }
    #[test]
    fn indirect1() {
        let main_content = r#"
        Context cx = Context.create();

        Source source1 = Source.newBuilder("python", new File("TestSamples/pyprint.py")).build();
        context.eval(source1);
        "#;
        let file_content = main_wrap(main_content);
        let tree = crate::tree_sitter_utils::parse(&file_content);
        let cst = crate::tree_sitter_utils::into(tree.as_ref(), &file_content);
        let builder = &JavaBuilder::init(cst);

        // TODO extract into find_polyglot_uses
        println!("INDIRECT1");
        let tree = tree.as_ref().unwrap();
        dbg!(tree.root_node().to_sexp());
        let class = tree.root_node().child(5).unwrap();
        let meth = class.child(3).unwrap().child(1).unwrap();
        let meth_body = &meth.child(4).unwrap();
        dbg!(meth_body.to_sexp());
        let poly_eval = meth_body.child(2).unwrap().child(0).unwrap();
        dbg!(poly_eval.to_sexp());
        let r#use = builder.try_compute_polyglot_use(&poly_eval);

        assert_eq!(r#use, None);
    }
    #[test]
    fn indirect2() {
        let main_content = r#"
        Context cx = Context.create();

        File file1 = new File("TestSamples/pyprint.py");
        Source source1 = Source.newBuilder("python", file1).build();
        context.eval(source1);
        "#;
        let file_content = main_wrap(main_content);
        let tree = crate::tree_sitter_utils::parse(&file_content);
        let cst = crate::tree_sitter_utils::into(tree.as_ref(), &file_content);
        let builder = &JavaBuilder::init(cst);

        // TODO extract into find_polyglot_uses
        println!("INDIRECT2");
        let tree = tree.as_ref().unwrap();
        dbg!(tree.root_node().to_sexp());
        let class = tree.root_node().child(5).unwrap();
        let meth = class.child(3).unwrap().child(1).unwrap();
        let meth_body = &meth.child(4).unwrap();
        dbg!(meth_body.to_sexp());
        let poly_eval = meth_body.child(2).unwrap().child(0).unwrap();
        dbg!(poly_eval.to_sexp());
        // let r#use: Option<Result<UnSolvedPolyglotUse, crate::building::AnaError>> = builder.try_compute_polyglot_use(&poly_eval);
        // assert_eq!(r#use, None);
        let finding = builder.find_polyglot_uses();
        dbg!(finding);
    }

    //todo
    //faire test for u in find_polyglot_uses
    //commencer par petits cas avec solved(s) puis faire cas importants avec eval(e) et eval(s)
    //faire des assert equals dans les tests pour vérifier qu'on a bien les valeurs qu'on veut

    //les eval E et eval S correspondent au todo à compléter
    //ce qui est polyglotte c'est pas la ligne entière de code mais juste le eval(s)

    //pour les noms de déclaration, possibilité de modifier les enums et d'en rajouter
    //ils ne sont pas spécialement adaptés
    //pas obligés d'avoir les bon noms pour les bons use il faut juste prendre tous les cas poluyglottes
}
