import ChildHome from './features/home/ChildHome';
import { tauriKidOSApi, type KidOSApi } from './lib/kidos-api';

export default function App({ api = tauriKidOSApi }: { api?: KidOSApi }) {
  return <ChildHome api={api} />;
}
