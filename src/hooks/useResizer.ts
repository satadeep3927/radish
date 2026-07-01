import { createSignal, onMount, onCleanup } from "solid-js";

export function useResizer(initialWidth: number = 240, minWidth: number = 150, maxWidth: number = 600, sidebarWidth: number = 200) {
  const [width, setWidth] = createSignal(initialWidth);
  const [isResizing, setIsResizing] = createSignal(false);

  const handlePointerDown = () => {
    setIsResizing(true);
    document.body.style.cursor = 'col-resize';
  };

  const handlePointerMove = (e: PointerEvent) => {
    if (isResizing()) {
      setWidth(Math.max(minWidth, Math.min(maxWidth, e.clientX - sidebarWidth)));
    }
  };

  const handlePointerUp = () => {
    setIsResizing(false);
    document.body.style.cursor = '';
  };

  onMount(() => {
    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp);
  });

  onCleanup(() => {
    window.removeEventListener('pointermove', handlePointerMove);
    window.removeEventListener('pointerup', handlePointerUp);
  });

  return {
    width,
    isResizing,
    handlePointerDown,
  };
}
