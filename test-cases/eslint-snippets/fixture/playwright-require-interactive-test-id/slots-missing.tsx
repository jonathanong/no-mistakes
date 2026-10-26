<>
  <Button asChild>
    <a href="/missing" />
  </Button>
  <Button asChild data-pw={dynamicId}>
    <a href="/dynamic" />
  </Button>
  <Button asChild {...props} asChild={true}>
    <a href="/spread" />
  </Button>
  <Button asChild data-pw>
    <button />
  </Button>
  <Button asChild data-pw={`prefix-${dynamicId}`}>
    <a href="/template" />
  </Button>
  <Button asChild data-pw="earlier" data-pw={dynamicId}>
    <a href="/duplicate" />
  </Button>
  <Button asChild data-pw={123}>
    <button />
  </Button>
  <Button data-pw="earlier" {...props} asChild>
    <a href="/overridden-id" />
  </Button>
</>;
