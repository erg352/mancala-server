import './App.css'

async function readJson(path) {
    return fetch(path).then(response => {
        return response.json();
    }).then(data => {
        console.log(data);
    });
}

function App() {
    function handleClick() {
        const data = readJson("http://localhost:8080/api/display/show_bots").catch(_ => []);

        const items = data.map(bot => <li>
            key={bot[1]},
            {bot[0]}
        </li>
        );

        return <ol>{items}</ol>
    }


    return (
        <h1>Hello!</h1>,
        <button onClick={handleClick}></button>
    )
}

export default App
