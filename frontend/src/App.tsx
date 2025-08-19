import CssBaseline from '@mui/material/CssBaseline';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { Provider, createStore } from 'jotai';
import { BrowserRouter as Router } from 'react-router-dom';

const theme = createTheme({});

// Jotai store instance
const store = createStore();

function App() {
  return (
    <Provider store={store}>
      <ThemeProvider theme={theme}>
        <CssBaseline />
        <Router></Router>
      </ThemeProvider>
    </Provider>
  );
}

export default App;
