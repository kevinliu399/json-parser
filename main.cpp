#include <iostream>
#include <fstream>
#include <string>
#include <stack>
using namespace std;

enum State
{
    START,
    EXPECT_KEY,
    IN_KEY,
    EXPECT_COLON,
    EXPECT_VALUE,
    IN_VALUE,
    END
};

bool isValidJSON(ifstream& file)
{
    State state = START;
    char ch;
    bool inQuotes = false;

    while (file.get(ch))
    {
        switch (state)
        {
            case START:
                if (ch == '{') state = EXPECT_KEY;
                else if (!isspace(ch)) return false;
                break;

            case EXPECT_KEY:
                if (ch == '"') {
                    state = IN_KEY;
                    inQuotes = true;
                }
                else if (ch == '}' && state == EXPECT_KEY) state = END;
                else if (!isspace(ch)) return false;
                break;

            case IN_KEY:
                if (ch == '"' && !inQuotes) {
                    state = EXPECT_COLON;
                    inQuotes = false;
                }
                else if (ch == '\\') inQuotes = !inQuotes;
                break;

            case EXPECT_COLON:
                if (ch == ':') state = EXPECT_VALUE;
                else if (!isspace(ch)) return false;
                break;

            case EXPECT_VALUE:
                if (ch == '"') {
                    state = IN_VALUE;
                    inQuotes = true;
                }
                else if (!isspace(ch)) return false;
                break;

            case IN_VALUE:
                if (ch == '"' && !inQuotes) {
                    state = EXPECT_KEY;
                    inQuotes = false;
                }
                else if (ch == '\\') inQuotes = !inQuotes;
                break;

            case END:
                if (!isspace(ch)) return false;
                break;
        }
    }

    return state == END;
}

int main()
{
    string filename = "test.txt";  // Change this to your test file
    ifstream file(filename);
    
    if (!file.is_open())
    {
        cout << "Unable to open file: " << filename << endl;
        return 1;
    }

    bool isValid = isValidJSON(file);
    cout << "Is valid JSON: " << (isValid ? "Yes" : "No") << endl;

    file.close();
    return 0;
}