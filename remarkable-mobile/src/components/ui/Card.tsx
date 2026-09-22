// src/components/ui/Card.tsx
import React from 'react';
import { View, StyleSheet, ViewStyle, Pressable } from 'react-native';

interface CardProps {
  children: React.ReactNode;
  style?: ViewStyle;
  onPress?: () => void;
  onLongPress?: () => void;
  selected?: boolean;
}

export function Card({ children, style, onPress, onLongPress, selected }: CardProps) {
  const content = (
    <View style={[styles.card, selected && styles.selected, style]}>
      {children}
    </View>
  );

  if (onPress || onLongPress) {
    return (
      <Pressable
        onPress={onPress}
        onLongPress={onLongPress}
        style={({ pressed }) => [pressed && styles.pressed]}
      >
        {content}
      </Pressable>
    );
  }

  return content;
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  selected: {
    borderWidth: 2,
    borderColor: '#000',
  },
  pressed: {
    opacity: 0.9,
  },
});
