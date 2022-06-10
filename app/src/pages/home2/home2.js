import React, { useState } from 'react';
import memraLogo from '../../logo3.svg';
import '../home/style.css';

function Home() {
    const [showMenu, setShowMenu] = useState(false);

    let menu = (
        <div className="togglemenu">
            <div className="showmenu" onClick={() => setShowMenu(!showMenu)}>
                <div className="menubutton-line" />
                <div className="menubutton-line" />
                <div className="menubutton-line" />
            </div>
        </div>
    );
    if (showMenu) {
        menu = (
            <div className="menu">
                <div className="hidemenu" onClick={() => setShowMenu(!showMenu)}>
                    <div className="menubutton-line" />
                    <div className="menubutton-line" />
                    <div className="menubutton-line" />
                </div>
                <div className="menuitem"><p>Home</p></div>
                <div className="menuitem"><p>Courses</p></div>
                <div className="menuitem"><p>Decks</p></div>
                <div className="menuitem"><p>Profile</p></div>
            </div>
        );
    }

    return (
        <div className="wrapper">
            {menu}
            <div className="container">
                <img src={memraLogo} className="logo" alt="Memra logo"/>
                <h1>Home</h1>
                <div className="row">
                    <div className="square">
                        <div
                            className="small_square"/>
                    </div>
                    <div className="square"/>
                    <div className="square"/>
                </div>
            </div>
        </div>
    );
}

export default Home;
