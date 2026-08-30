using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AbilityActivated_EUNeverGiveIn
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.AbilityActivated_EUNeverGiveIn); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.AbilityActivated_EUNeverGiveIn)obj;
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
            //  Serialize Radius
            s.Write(value.Radius);
            //  Serialize PlayerPowerPosition
            s.Write(value.PlayerPowerPosition);
            //  Serialize AbilityId
            s.Write(value.AbilityId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.AbilityActivated_EUNeverGiveIn)) as Rts.CnC.Messages.Client.AbilityActivated_EUNeverGiveIn;
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
            //  Deserialize Radius
            s.Read(out value.Radius);
            //  Deserialize PlayerPowerPosition
            s.Read(out value.PlayerPowerPosition);
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);

            return value;
        }
        
    }
}
