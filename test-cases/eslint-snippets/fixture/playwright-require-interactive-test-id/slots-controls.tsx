<>
  <Button asChild={false} data-pw="parent"><a href="/false" /></Button>
  <Button asChild={enabled} data-pw="parent"><a href="/unknown" /></Button>
  <Button asChild={null} data-pw="parent"><a href="/null" /></Button>
  <Button asChild={0} data-pw="parent"><a href="/zero" /></Button>
  <Button asChild="" data-pw="parent"><a href="/empty" /></Button>
  <Button asChild={``} data-pw="parent"><a href="/empty-template" /></Button>
  <Button data-pw="parent"><a href="/absent" /></Button>
  <Button asChild data-pw="parent" {...props}><a href="/overridden" /></Button>
  <Button asChild data-pw="parent" asChild={false}><a href="/last-false" /></Button>
  <Button asChild data-pw="parent"><a href="/first" /><button /></Button>
  <Button asChild data-pw="parent"><div><button /></div></Button>
  <Button asChild data-pw="parent">Text<a href="/text" /></Button>
  <Button asChild data-pw="parent">{other}<a href="/expression" /></Button>
  <Button asChild data-pw="parent"><><a href="/fragment" /></></Button>
  <Unknown asChild data-pw="parent"><a href="/unconfigured" /></Unknown>
  <button asChild data-pw="parent"><a href="/native" /></button>
  <Button asChild data-pw="parent"><a {...props} href="/child-spread" /></Button>
  <Button asChild data-pw="parent">{<a href="/wrapped" />}</Button>
  {/* Same-line spaces are actual React text children, unlike newline formatting. */}
  <Button asChild data-pw="parent"> <a href="/spaces" /> </Button>
  <Button asChild data-pw="parent"><a href="/outer"><span onClick={click} /></a></Button>
  <Button asChild><a href="/missing-many" /><button /></Button>
</>;
