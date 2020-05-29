import React from 'react'
import { Button, TextField } from '@material-ui/core'


export default class LoginScreen extends React.Component {

  state = {
    token: '',
  }

  render() {
    return (
      <section style={{display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh'}}>
        <form>
          <TextField type='password' placeholder='Token or password' label='Token or password' value={this.state.token} onChange={e => {
            e.preventDefault()
            this.setState({token: e.target.value})
          }}/>
          <br/>
          <br/>
          <Button fullWidth type='submit' variant='contained' color='primary' onClick={e => {
            e.preventDefault()
            submit()
          }}>Login</Button>
        </form>
      </section>
    )
  }
}