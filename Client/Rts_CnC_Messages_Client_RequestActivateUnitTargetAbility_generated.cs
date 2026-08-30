using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestActivateUnitTargetAbility
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestActivateUnitTargetAbility); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestActivateUnitTargetAbility)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize array UnitIds
            Rts.Serialization.Reference.Write(s, value.UnitIds, () =>
            {
                s.WriteVarInt32(value.UnitIds.Length);
                for(int i = 0 ; i < value.UnitIds.Length ; ++i)
                {
                    s.Write(value.UnitIds[i]);
                }
            });
            //  Serialize TargetPlayerId
            s.Write(value.TargetPlayerId);
            //  Serialize TargetEntityId
            s.Write(value.TargetEntityId);
            //  Serialize AbilityName
            s.Write(value.AbilityName);
            //  Serialize ModifierFlags
            s.WriteEnum(value.ModifierFlags);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestActivateUnitTargetAbility)) as Rts.CnC.Messages.Client.RequestActivateUnitTargetAbility;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize array UnitIds
            Rts.Serialization.Reference.Read(s, out value.UnitIds, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize TargetPlayerId
            s.Read(out value.TargetPlayerId);
            //  Deserialize TargetEntityId
            s.Read(out value.TargetEntityId);
            //  Deserialize AbilityName
            s.Read(out value.AbilityName);
            //  Deserialize ModifierFlags
            s.ReadEnum(out value.ModifierFlags);

            return value;
        }
        
    }
}
