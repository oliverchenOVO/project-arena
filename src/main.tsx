import { createRoot } from 'react-dom/client';
import App from './Arena';
import {LocaleProvider} from './i18n';
import './styles.css';
import './compact.css';
createRoot(document.getElementById('root')!).render(<LocaleProvider><App/></LocaleProvider>);
import './presentation.css';
import './domains.css';
