import { createApp } from "vue";
import App from "./App.vue";
import PreviewWindowPage from "./pages/PreviewWindowPage.vue";
import "./style.css";

const params = new URLSearchParams(window.location.search);
const rootComponent = params.get("window") === "preview" ? PreviewWindowPage : App;

createApp(rootComponent).mount("#app");
