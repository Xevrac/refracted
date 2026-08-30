using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_TargetedLocationsAbilityActivated
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.TargetedLocationsAbilityActivated); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.TargetedLocationsAbilityActivated)obj;
            //  Serialize Radius
            s.Write(value.Radius);
            //  Serialize array Positions
            Rts.Serialization.Reference.Write(s, value.Positions, () =>
            {
                s.WriteVarInt32(value.Positions.Length);
                for(int i = 0 ; i < value.Positions.Length ; ++i)
                {
                    s.Write(value.Positions[i]);
                }
            });
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize AbilityId
            s.Write(value.AbilityId);
            //  Serialize MillisecondsToReenable
            s.Write(value.MillisecondsToReenable);
            //  Serialize DelayBeforeActivation
            s.Write(value.DelayBeforeActivation);
            //  Serialize Flags
            s.Write(value.Flags);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.TargetedLocationsAbilityActivated)) as Rts.CnC.Messages.Client.TargetedLocationsAbilityActivated;
            //  Deserialize Radius
            s.Read(out value.Radius);
            //  Deserialize array Positions
            Rts.Serialization.Reference.Read(s, out value.Positions, () =>
            {
                int length = s.ReadVarInt32();
                SlimMath.Vector3[] tmp = new SlimMath.Vector3[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);
            //  Deserialize MillisecondsToReenable
            s.Read(out value.MillisecondsToReenable);
            //  Deserialize DelayBeforeActivation
            s.Read(out value.DelayBeforeActivation);
            //  Deserialize Flags
            s.Read(out value.Flags);

            return value;
        }
        
    }
}
