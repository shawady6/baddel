// Tabs that were already open when the extension was installed or reloaded don't get
// the content script automatically, so inject it once.
async function injectIntoOpenTabs() {
  const tabs = await chrome.tabs.query({ url: ["http://*/*", "https://*/*"] });
  for (const tab of tabs) {
    if (!tab.id) continue;
    try {
      await chrome.scripting.executeScript({
        target: { tabId: tab.id, allFrames: true },
        files: ["shared.js", "content.js"]
      });
    } catch (e) {
      // Protected pages (Web Store, PDF viewer, ...) refuse injection; ignore.
    }
  }
}

chrome.runtime.onInstalled.addListener((details) => {
  if (details.reason === "install" || details.reason === "update") injectIntoOpenTabs();
});
