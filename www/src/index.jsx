import { createRoot } from 'react-dom/client';

import App from './App';
import { Provider } from './Provider';

import './styles/index.css';

const container = document.getElementById('app');
const root = createRoot(container);
root.render(<Provider><App /></Provider>);
