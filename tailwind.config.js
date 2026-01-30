/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        bg: {
          primary: '#0d0d0f',
          secondary: '#16161a',
          tertiary: '#1c1c22',
          card: 'rgba(28, 28, 34, 0.6)',
          glass: 'rgba(255, 255, 255, 0.03)',
        },
        border: {
          DEFAULT: 'rgba(255, 255, 255, 0.08)',
          hover: 'rgba(255, 255, 255, 0.12)',
          'glass': 'rgba(255, 255, 255, 0.05)',
        },
        text: {
          primary: '#f5f5f7',
          secondary: 'rgba(255, 255, 255, 0.6)',
          tertiary: 'rgba(255, 255, 255, 0.4)',
        },
        status: {
          running: '#34d399',
          waiting: '#fbbf24',
          completed: '#60a5fa',
          blocked: '#f87171',
        },
      },
      backdropBlur: {
        xs: '2px',
        glass: '8px',
      },
      borderRadius: {
        'card': '16px',
        'button': '8px',
      },
      animation: {
        'pulse-slow': 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'fade-in': 'fadeIn 300ms ease-out',
        'slide-down': 'slideDown 300ms ease-out',
        'slide-up': 'slideUp 300ms ease-out',
        'scale-in': 'scaleIn 300ms ease-out',
        'message-in': 'messageIn 300ms ease-out',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideDown: {
          '0%': { opacity: '0', transform: 'translateY(-10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        slideUp: {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        scaleIn: {
          '0%': { opacity: '0', transform: 'scale(0.95)' },
          '100%': { opacity: '1', transform: 'scale(1)' },
        },
        messageIn: {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        pulse: {
          '0%, 100%': { opacity: '1', transform: 'scale(1)' },
          '50%': { opacity: '0.5', transform: 'scale(0.8)' },
        },
      },
    },
  },
  plugins: [],
};
