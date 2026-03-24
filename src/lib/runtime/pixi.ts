import * as PIXI from "pixi.js";

window.PIXI = PIXI;

export type RendererBundle = {
  app: PIXI.Application;
  root: PIXI.Container;
  destroy: () => void;
};

export function createRenderer(
  host: HTMLElement,
  background: string,
): RendererBundle {
  host.innerHTML = "";

  const app = new PIXI.Application({
    resizeTo: host,
    antialias: true,
    autoDensity: true,
    backgroundAlpha: 0,
  });
  host.appendChild(app.view as HTMLCanvasElement);

  const root = new PIXI.Container();
  root.position.set(app.screen.width / 2, app.screen.height / 2);
  app.stage.addChild(root);

  host.style.background = background;
  const observer = new ResizeObserver(() => {
    root.position.set(app.screen.width / 2, app.screen.height / 2);
  });
  observer.observe(host);

  return {
    app,
    root,
    destroy: () => {
      observer.disconnect();
      app.destroy(true, true);
      host.innerHTML = "";
    },
  };
}

export function syncBackground(host: HTMLElement, background: string): void {
  host.style.background = background;
}

export function attachPanAndZoom(
  view: HTMLCanvasElement,
  container: PIXI.Container,
  zoom: { value: number; onChange?: (value: number) => void },
): () => void {
  let isDragging = false;
  let lastX = 0;
  let lastY = 0;

  const onPointerDown = (event: PointerEvent) => {
    isDragging = true;
    lastX = event.clientX;
    lastY = event.clientY;
    view.setPointerCapture(event.pointerId);
  };

  const onPointerMove = (event: PointerEvent) => {
    if (!isDragging) {
      return;
    }
    container.position.x += event.clientX - lastX;
    container.position.y += event.clientY - lastY;
    lastX = event.clientX;
    lastY = event.clientY;
  };

  const onPointerUp = (event: PointerEvent) => {
    isDragging = false;
    if (view.hasPointerCapture(event.pointerId)) {
      view.releasePointerCapture(event.pointerId);
    }
  };

  const onWheel = (event: WheelEvent) => {
    event.preventDefault();
    zoom.value = Math.min(3, Math.max(0.2, zoom.value + (event.deltaY > 0 ? -0.06 : 0.06)));
    container.scale.set(zoom.value);
    zoom.onChange?.(zoom.value);
  };

  view.addEventListener("pointerdown", onPointerDown);
  view.addEventListener("pointermove", onPointerMove);
  view.addEventListener("pointerup", onPointerUp);
  view.addEventListener("pointerleave", onPointerUp);
  view.addEventListener("wheel", onWheel, { passive: false });

  return () => {
    view.removeEventListener("pointerdown", onPointerDown);
    view.removeEventListener("pointermove", onPointerMove);
    view.removeEventListener("pointerup", onPointerUp);
    view.removeEventListener("pointerleave", onPointerUp);
    view.removeEventListener("wheel", onWheel);
  };
}
