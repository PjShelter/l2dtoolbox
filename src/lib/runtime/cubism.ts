declare global {
  interface Window {
    PIXI?: unknown;
  }
}

let cubismBootstrap: Promise<void> | null = null;

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
