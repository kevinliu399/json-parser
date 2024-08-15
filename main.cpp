#include <iostream>
#include <fstream>
#include <string>
#include <stack>
using namespace std;

enum State
{
    START,
    KEY,
    COLON,
    VALUE,
};

// use & to pass by reference
bool matchingBrackets(ifstream& file)
{
    stack<char> bracketStack;
    int validBracket = 0;
    char ch;

    while (file.get(ch))
    {
        if (ch == '{')
        {
            bracketStack.push(ch);
        }
        else if (ch == '}')
        {
            if (bracketStack.empty())
            {
                return false; // no matching opening bracket
            }
            else
            {
                bracketStack.pop();
                validBracket++;
            }
        }
    }

    return bracketStack.empty();
}

bool isValidJSON(ifstream& file);

int main()
{
    string line;
    ifstream myfile("test.txt"); // ifstream -> read from stream
    bool isMatching;
    if (myfile.is_open())
    {
        isMatching = matchingBrackets(myfile);
        cout << "Matching brackets: " << isMatching << endl;
        myfile.close();
    }

    else
        cout << "Unable to open file";

    return 0;
}