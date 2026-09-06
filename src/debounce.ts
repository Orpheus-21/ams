/// A trailing debounce that can also be cancelled or run immediately.
///
/// The editor debounces two independent things — recompiling and following the
/// cursor — and both need "cancel whatever is pending". Doing that with raw
/// timer variables meant a `clearTimeout` at every call site, and forgetting
/// one is exactly how new/open ended up compiling twice.
export interface Debounced {
  (): void;
  cancel(): void;
  /// Cancels anything pending and runs now.
  now(): void;
}

export function debounce(fn: () => void, delayMs: number): Debounced {
  let timer: number | undefined;

  const trigger = (() => {
    window.clearTimeout(timer);
    timer = window.setTimeout(fn, delayMs);
  }) as Debounced;

  trigger.cancel = () => window.clearTimeout(timer);
  trigger.now = () => {
    trigger.cancel();
    fn();
  };

  return trigger;
}
