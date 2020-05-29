import React from 'react';
import './App.css';

import 'typeface-roboto';
import LoginScreen from './screens/LoginScreen';

function submit(token) {
  console.log(token)
}

function App() {
  return (
    <LoginScreen/>
  )
}

export default App;
