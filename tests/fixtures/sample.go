package main

import "fmt"

type Greeter struct {
	Name string
}

func (g *Greeter) Greet() string {
	return fmt.Sprintf("Hello, %s!", g.Name)
}

func helper(x int) int {
	return x * 2
}

func main() {
	g := &Greeter{Name: "World"}
	msg := g.Greet()
	fmt.Println(msg)
	result := helper(42)
	fmt.Println(result)
}
