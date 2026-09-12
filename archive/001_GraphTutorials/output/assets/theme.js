/**
 * MDRender Client-side Theme & Interaction Script
 */

(function () {
  'use strict';

  const STORAGE_THEME_KEY = 'mdrender-theme';
  const STORAGE_MODE_KEY = 'mdrender-mode';

  // 1. Theme and Mode Management
  function getSystemMode() {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }

  function getSavedTheme() {
    return localStorage.getItem(STORAGE_THEME_KEY) || 'github';
  }

  function getSavedMode() {
    return localStorage.getItem(STORAGE_MODE_KEY) || 'auto';
  }

  function applyTheme(theme, mode) {
    const effectiveMode = mode === 'auto' ? getSystemMode() : mode;
    const root = document.documentElement;

    root.setAttribute('data-theme', theme);
    root.setAttribute('data-mode', effectiveMode);

    // Update select element if present
    const themeSelect = document.getElementById('theme-select');
    if (themeSelect && themeSelect.value !== theme) {
      themeSelect.value = theme;
    }

    // Update mode button label/icon
    const modeBtn = document.getElementById('mode-toggle');
    if (modeBtn) {
      let icon = '💻 Auto';
      if (mode === 'light') icon = '☀️ Light';
      if (mode === 'dark') icon = '🌙 Dark';
      modeBtn.textContent = icon;
      modeBtn.setAttribute('title', `Current Mode: ${mode.toUpperCase()} (Click to toggle)`);
    }
  }

  function cycleMode(currentMode) {
    if (currentMode === 'auto') return 'light';
    if (currentMode === 'light') return 'dark';
    return 'auto';
  }

  // 2. Setup Code Copy Buttons
  function setupCodeCopy() {
    const copyButtons = document.querySelectorAll('.copy-btn');
    copyButtons.forEach((btn) => {
      btn.addEventListener('click', async () => {
        const wrapper = btn.closest('.code-block-wrapper');
        const codeElement = wrapper ? wrapper.querySelector('pre code') : null;
        if (!codeElement) return;

        const textToCopy = codeElement.innerText;
        try {
          await navigator.clipboard.writeText(textToCopy);
          const originalText = btn.textContent;
          btn.textContent = 'Copied!';
          btn.classList.add('copied');
          setTimeout(() => {
            btn.textContent = originalText;
            btn.classList.remove('copied');
          }, 1800);
        } catch (err) {
          console.error('Failed to copy to clipboard', err);
        }
      });
    });
  }

  // 3. Setup Table of Contents Active Section Tracker
  function setupTocHighlight() {
    const tocLinks = document.querySelectorAll('.toc-link');
    if (tocLinks.length === 0) return;

    const headings = [];
    tocLinks.forEach((link) => {
      const id = link.getAttribute('href');
      if (id && id.startsWith('#')) {
        const target = document.getElementById(id.substring(1));
        if (target) headings.push({ element: target, link });
      }
    });

    if (headings.length === 0) return;

    let activeLink = null;
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const id = '#' + entry.target.id;
            tocLinks.forEach((link) => {
              if (link.getAttribute('href') === id) {
                if (activeLink) activeLink.classList.remove('active');
                link.classList.add('active');
                activeLink = link;
              }
            });
          }
        });
      },
      {
        rootMargin: '0px 0px -60% 0px',
        threshold: 0.1,
      }
    );

    headings.forEach((h) => observer.observe(h.element));
  }

  // 4. Mobile Sidebar Drawer
  function setupMobileSidebar() {
    const toggleBtn = document.querySelector('.mobile-nav-toggle');
    const sidebar = document.querySelector('.sidebar');
    const backdrop = document.querySelector('.sidebar-backdrop');

    if (!toggleBtn || !sidebar || !backdrop) return;

    function toggleSidebar() {
      sidebar.classList.toggle('open');
      backdrop.classList.toggle('open');
    }

    toggleBtn.addEventListener('click', toggleSidebar);
    backdrop.addEventListener('click', toggleSidebar);
  }

  // 5. Sidebar Filter / Search
  function setupSearchFilter() {
    const searchInput = document.querySelector('.search-input');
    if (!searchInput) return;

    searchInput.addEventListener('input', (e) => {
      const query = e.target.value.toLowerCase().trim();
      const navItems = document.querySelectorAll('.nav-item');

      navItems.forEach((item) => {
        const text = item.textContent.toLowerCase();
        if (text.includes(query)) {
          item.style.display = '';
        } else {
          item.style.display = 'none';
        }
      });
    });
  }

  // Initialize all components on DOMContentLoaded
  document.addEventListener('DOMContentLoaded', () => {
    let currentTheme = getSavedTheme();
    let currentMode = getSavedMode();

    applyTheme(currentTheme, currentMode);

    // Watch for OS dark mode changes
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (getSavedMode() === 'auto') {
        applyTheme(getSavedTheme(), 'auto');
      }
    });

    // Theme selector
    const themeSelect = document.getElementById('theme-select');
    if (themeSelect) {
      themeSelect.addEventListener('change', (e) => {
        currentTheme = e.target.value;
        localStorage.setItem(STORAGE_THEME_KEY, currentTheme);
        applyTheme(currentTheme, currentMode);
      });
    }

    // Mode toggle button
    const modeBtn = document.getElementById('mode-toggle');
    if (modeBtn) {
      modeBtn.addEventListener('click', () => {
        currentMode = cycleMode(currentMode);
        localStorage.setItem(STORAGE_MODE_KEY, currentMode);
        applyTheme(currentTheme, currentMode);
      });
    }

    setupCodeCopy();
    setupTocHighlight();
    setupMobileSidebar();
    setupSearchFilter();
  });
})();
