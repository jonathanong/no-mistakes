// Opaque applications alone must not allocate receiver capture/call indexes.
@opaque
class Decorated {
  @opaque method() {}
  static value: number = 1;
  static missing: number;
  @opaque static accessor initialized: number = 1;
  @opaque static accessor plain = 1;
  @opaque static accessor missingAccessor: number;
}
function opaque() {}

(class { @opaque method() {} static callable = () => {}; });

@(factory().decorate)
class Dynamic {}
function factory() { return { decorate: opaque }; }
