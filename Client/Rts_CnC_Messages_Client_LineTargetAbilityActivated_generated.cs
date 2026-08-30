using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_LineTargetAbilityActivated
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.LineTargetAbilityActivated); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.LineTargetAbilityActivated)obj;
            //  Serialize PlayerPowerPosition
            s.Write(value.PlayerPowerPosition);
            //  Serialize PlayerPowerDirection
            s.Write(value.PlayerPowerDirection);
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
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.LineTargetAbilityActivated)) as Rts.CnC.Messages.Client.LineTargetAbilityActivated;
            //  Deserialize PlayerPowerPosition
            s.Read(out value.PlayerPowerPosition);
            //  Deserialize PlayerPowerDirection
            s.Read(out value.PlayerPowerDirection);
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
