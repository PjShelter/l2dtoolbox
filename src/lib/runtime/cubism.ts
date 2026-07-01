declare global {
  interface Window {
    PIXI?: unknown;
  }
}

let cubismBootstrap: Promise<void> | null = null;
let live2dModuleBootstrap: Promise<typeof import("pixi-live2d-display-webgal")> | null =
  null;

export async function ensureCubismRuntime(): Promise<void> {
  if (cubismBootstrap) {
    return cubismBootstrap;
  }

  cubismBootstrap = (async () => {
    await injectScript("lib/live2d.min.js");
    await injectScript("lib/live2dcubismcore.min.js");
  })();

  return cubismBootstrap;
}

export async function loadLive2DModule(): Promise<
  typeof import("pixi-live2d-display-webgal")
> {
  if (live2dModuleBootstrap) {
    return live2dModuleBootstrap;
  }

  live2dModuleBootstrap = (async () => {
    await ensureCubismRuntime();
    const pixi = await import("pixi.js");
    window.PIXI = pixi;

    const live2d = await import("pixi-live2d-display-webgal");
    live2d.Live2DModel.registerTicker(pixi.Ticker);
    return live2d;
  })();

  return live2dModuleBootstrap;
}

async function injectScript(src: string): Promise<void> {
  const existing = document.querySelector<HTMLScriptElement>(
    `script[data-live2d-src="${src}"], script[data-cubism-src="${src}"]`,
  );
  if (existing) {
    await waitForScript(existing);
    return;
  }

  const script = document.createElement("script");
  script.src = src;
  script.async = true;
  script.dataset.cubismSrc = src;
  script.dataset.live2dSrc = src;
  document.head.appendChild(script);

  await waitForScript(script);
}

function waitForScript(script: HTMLScriptElement): Promise<void> {
  return new Promise((resolve, reject) => {
    if (script.dataset.loaded === "true") {
      resolve();
      return;
    }

    script.addEventListener(
      "load",
      () => {
        script.dataset.loaded = "true";
        resolve();
      },
      { once: true },
    );
    script.addEventListener(
      "error",
      () => reject(new Error(`Failed to load runtime script: ${script.src}`)),
      { once: true },
    );
  });
}
