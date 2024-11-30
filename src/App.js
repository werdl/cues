import logo from './logo.svg';
import './App.css';
import { invoke } from '@tauri-apps/api/core';

function App() {
  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <input type="text" id="verb" />
        <input type="text" id="arg1" />
        <input type="text" id="arg2" />
        <button onClick={() => {
          // only append an argument if it is not empty
          let args = [];
          if (document.getElementById('arg1').value !== '') {
            args.push(document.getElementById('arg1').value);
          }

          if (document.getElementById('arg2').value !== '') {
            args.push(document.getElementById('arg2').value);
          }


          invoke('parse_command', {
            verb: document.getElementById('verb').value,
            args: args
          }).then(response => {
            console.log(response);
          })
        }}>Calculate</button>
        <a
          className="App-link"
          href="https://reactjs.org"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn React
        </a>
      </header>
    </div>
  );
}

export default App;
