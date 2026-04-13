import { useState, useRef, useEffect } from "react";

interface SearchableSelectProps<T> {
  options: T[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  searchFields: (keyof T)[];
  displayFormatter: (item: T) => string;
  getItemValue: (item: T) => string;
  notFoundMessage?: string;
  label?: string;
  required?: boolean;
}

export function SearchableSelect<T>({
  options,
  value,
  onChange,
  placeholder = "Buscar...",
  searchFields,
  displayFormatter,
  getItemValue,
  notFoundMessage = "No se encontraron resultados",
  label,
  required,
}: SearchableSelectProps<T>) {
  const [searchTerm, setSearchTerm] = useState("");
  const [isOpen, setIsOpen] = useState(false);
  const [filteredOptions, setFilteredOptions] = useState<T[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Filter options based on search term
  useEffect(() => {
    if (!searchTerm.trim()) {
      setFilteredOptions(options.slice(0, 20)); // Show first 20 when empty
      return;
    }

    const term = searchTerm.toLowerCase();
    const filtered = options.filter((option) => {
      return searchFields.some((field) => {
        const fieldValue = option[field];
        if (typeof fieldValue === "string") {
          return fieldValue.toLowerCase().includes(term);
        }
        return false;
      });
    });
    setFilteredOptions(filtered.slice(0, 20));
  }, [searchTerm, options, searchFields]);

  // Get selected item for display
  const selectedItem = options.find((opt) => getItemValue(opt) === value);

  // Handle selection
  const handleSelect = (item: T) => {
    onChange(getItemValue(item));
    setSearchTerm("");
    setIsOpen(false);
    inputRef.current?.blur();
  };

  // Handle clear
  const handleClear = () => {
    onChange("");
    setSearchTerm("");
    setIsOpen(false);
  };

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node) &&
        inputRef.current &&
        !inputRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  return (
    <div className="relative" ref={dropdownRef}>
      {label && (
        <label className="block text-sm font-medium text-gray-700 mb-1">
          {label} {required && <span className="text-red-500">*</span>}
        </label>
      )}
      
      {/* Selected value display */}
      {selectedItem && !searchTerm && (
        <div className="flex items-center justify-between px-3 py-2 border border-gray-300 rounded-lg bg-gray-50">
          <span className="text-gray-900 truncate">
            {displayFormatter(selectedItem)}
          </span>
          <button
            type="button"
            onClick={handleClear}
            className="ml-2 text-gray-400 hover:text-gray-600"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      )}

      {/* Search input */}
      {(!selectedItem || searchTerm) && (
        <div className="relative">
          <input
            ref={inputRef}
            type="text"
            value={searchTerm}
            onChange={(e) => {
              setSearchTerm(e.target.value);
              setIsOpen(true);
            }}
            onFocus={() => setIsOpen(true)}
            placeholder={selectedItem ? displayFormatter(selectedItem) : placeholder}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            required={required}
          />
          
          {/* Search icon */}
          <div className="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
            <svg className="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>

          {/* Dropdown */}
          {isOpen && (
            <div className="absolute z-10 w-full mt-1 bg-white border border-gray-200 rounded-lg shadow-lg max-h-60 overflow-auto">
              {filteredOptions.length === 0 ? (
                <div className="px-4 py-3 text-gray-500 text-sm">
                  {notFoundMessage}
                </div>
              ) : (
                <ul>
                  {filteredOptions.map((item, index) => {
                    const itemValue = getItemValue(item);
                    const isSelected = itemValue === value;
                    
                    // Highlight matching text
                    const displayText = displayFormatter(item);
                    const highlightedText = searchTerm
                      ? highlightMatch(displayText, searchTerm)
                      : displayText;

                    return (
                      <li
                        key={itemValue || index}
                        onClick={() => handleSelect(item)}
                        className={`px-4 py-3 cursor-pointer hover:bg-blue-50 ${
                          isSelected ? "bg-blue-100 text-blue-900" : ""
                        }`}
                      >
                        <div
                          dangerouslySetInnerHTML={{ __html: highlightedText }}
                        />
                        {itemValue && (
                          <div className="text-xs text-gray-400 mt-0.5">
                            ID: {itemValue.substring(0, 8)}...
                          </div>
                        )}
                      </li>
                    );
                  })}
                </ul>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// Helper to highlight matching text
function highlightMatch(text: string, searchTerm: string): string {
  if (!searchTerm) return text;
  
  const regex = new RegExp(`(${escapeRegExp(searchTerm)})`, "gi");
  return text.replace(regex, "<mark class='bg-yellow-200 font-semibold'>$1</mark>");
}

function escapeRegExp(string: string): string {
  return string.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
