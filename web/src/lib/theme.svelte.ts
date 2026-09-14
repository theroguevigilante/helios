export const themeState = $state({
    isDark: true
});

export function initTheme() {
    if (typeof window !== 'undefined') {
        const stored = localStorage.getItem('helios-theme');
        if (stored === 'light') {
            themeState.isDark = false;
        } else {
            themeState.isDark = true;
        }
        applyTheme();
    }
}

export function toggleTheme() {
    themeState.isDark = !themeState.isDark;
    if (typeof window !== 'undefined') {
        localStorage.setItem('helios-theme', themeState.isDark ? 'dark' : 'light');
        applyTheme();
    }
}

function applyTheme() {
    if (typeof document !== 'undefined') {
        if (themeState.isDark) {
            document.documentElement.classList.add('dark');
            document.querySelector('meta[name="theme-color"]')?.setAttribute('content', '#0a0a0f');
        } else {
            document.documentElement.classList.remove('dark');
            document.querySelector('meta[name="theme-color"]')?.setAttribute('content', '#f8fafc');
        }
        
        // Dispatch custom event for echarts or other imperative listeners
        window.dispatchEvent(new CustomEvent('theme-changed', { detail: { isDark: themeState.isDark }}));
    }
}
