import React from 'react';
import { Button, TextField } from '@material-ui/core'
import './App.css';

import 'typeface-roboto';

function submit(token) {
  console.log(token)
}

function App() {
  const [token, setToken] = React.useState('')
  return (
    <section style={{display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh'}}>
      <form>
        <TextField type='password' placeholder='Token or password' label='Token or password' value={token} onChange={e => {
          e.preventDefault()
          setToken(e.target.value)
        }}/>
        <br/>
        <br/>
        <Button fullWidth type='submit' variant='contained' color='primary' onClick={e => {
          e.preventDefault()
          submit()
        }}>Login</Button>
      </form>
    </section>
  );
}

export default App;
