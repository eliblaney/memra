import React, { useState } from 'react';
import memraLogo from '../../logo1.svg';
import tileImage from '../../anatomy_placeholder.jpg'
import './style.css';

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
						<div className="wrapper">
							<img src="memraLogo" className="profilephoto" alt="Profile image"/>
							<div className="menuitem"><p>Profile</p></div>
						</div>
						<div className="menuitem"><p>Home</p></div>
						<div className="menuitem"><p>Courses</p></div>
						<div className="menuitem"><p>Decks</p></div>
						<div className="menuitem"><p>Notifications</p></div>
						<div className="menuitem"><p>Settings</p></div>
					</div>
				);
		}

		return (
		<div className="wrapper">
			{menu}
			<div className="container">
				<img src={memraLogo} className="logo" alt="Memra logo"/>
				<h1 className="headers">Favorite Decks</h1>
				<div className="row">
					<div className="tile">
						<img src="tileImage" className="tileImage" alt="Image for tiles"/>
					</div>
					<div className="tile">
						<img src="tileImage" className="tileImage" alt="Image for tiles"/>
					</div>
					<div className="tile">
						<img src="tileImage" className="tileImage" alt="Image for tiles"/>
					</div>
				</div>
			</div>
		</div>
		);
}

export default Home;
