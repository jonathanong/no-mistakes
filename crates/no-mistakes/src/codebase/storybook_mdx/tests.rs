use super::extract_mdx_source;

#[test]
fn extracts_mdx_runtime_import_shapes() {
    let facts = extract_mdx_source(
        r#"
import "./setup.story";
import './single-quote.story';
import DefaultCard from "../components/DefaultCard";
import DefaultButton, { Button as RenamedButton, Link } from "../components/Button";
	import * as Cards from "../components/Cards";
	import { type Props, Panel } from "../components/Panel";
	import type { TypeOnlyCard } from "../components/TypeOnlyCard";
	import type DefaultTypeOnly from "../components/DefaultTypeOnly";
	import {
	  MultilineCard as RenamedMultilineCard,
	  OtherMultilineCard,
	} from "../components/MultilineCard";
	import ignored from "../components/Ignored"

	```tsx
	~~~
	import FencedExample from "../components/FencedExample";
	```

	~~~
	import TildeFence from "../components/TildeFence";
	~~~

	<DefaultCard />
	<DefaultButton />
	<RenamedButton />
	<Link />
	<Cards.Card />
	<Panel />
	<RenamedMultilineCard />
	{OtherMultilineCard}
		"#,
    );

    assert_eq!(facts.side_effect_imports.len(), 2);
    assert_eq!(facts.side_effect_imports[0].source, "./setup.story");
    assert_eq!(facts.side_effect_imports[1].source, "./single-quote.story");
    assert_eq!(
        facts
            .used_runtime_imports
            .iter()
            .map(|import| (
                import.imported.as_str(),
                import.local.as_str(),
                import.source.as_str(),
                import.namespace,
            ))
            .collect::<Vec<_>>(),
        vec![
            ("default", "DefaultCard", "../components/DefaultCard", false,),
            ("default", "DefaultButton", "../components/Button", false,),
            ("Button", "RenamedButton", "../components/Button", false,),
            ("Link", "Link", "../components/Button", false),
            ("*", "Cards", "../components/Cards", true),
            ("Panel", "Panel", "../components/Panel", false),
            (
                "MultilineCard",
                "RenamedMultilineCard",
                "../components/MultilineCard",
                false,
            ),
            (
                "OtherMultilineCard",
                "OtherMultilineCard",
                "../components/MultilineCard",
                false,
            ),
        ]
    );
}

#[test]
fn ignores_malformed_mdx_imports() {
    let facts = extract_mdx_source(
        r#"
import
import nope
	import Bad from nope;
	import  from "../empty-default";
	import {} from "../empty";
	import { Missing from "../bad";
	import {
	import Later from "../Later";
	<Later />
	const value = "import { Nope } from './Nope'";
	"#,
    );

    assert_eq!(facts.used_runtime_imports.len(), 1);
    assert_eq!(facts.used_runtime_imports[0].local, "Later");
    assert!(facts.side_effect_imports.is_empty());

    let mut imports = Vec::new();
    super::push_mdx_default_import(&mut imports, "", "../empty-default", 1);
    super::push_mdx_imports(&mut imports, "type Props", "../type-clause", 1);
    super::push_mdx_named_import(&mut imports, "Missing as ", "../bad-local", 1);
    assert!(imports.is_empty());

    let mut side_effects = Vec::new();
    super::push_mdx_import_line(&mut imports, &mut side_effects, "not a valid import", 1);
    assert!(imports.is_empty());
    assert!(side_effects.is_empty());
}
